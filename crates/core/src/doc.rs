//! The open document: the model, its file, and undo/redo. The window sends edit operations; each
//! runs on a copy and is kept only if the result lays out and passes `Model::check`, so a failed
//! edit leaves the model exactly as it was (Plan's pattern: ops in, whole-model snapshots for undo).

use crate::model::{Cond, Element, Load, LoadCase, Material, Model, Nozzle, Pipe, Project, Settings, Support};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// One item of the model, for adding or replacing.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Item {
    Element(Element),
    Pipe(Pipe),
    Material(Material),
    Cond(Cond),
    Support(Support),
    Nozzle(Nozzle),
    Load(Load),
    Case(LoadCase),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Op {
    /// Add an item; the model gives it a new id. An element is placed after the element `after`
    /// in the table (or at the end).
    Add { item: Item, after: Option<u32> },
    /// Replace the item that has the same id.
    Set { item: Item },
    /// Delete items by id. Deleting elements removes the nodes left without one, with the supports,
    /// nozzles and loads on them. A pipe, material or line conditions still in use is kept.
    Delete { ids: Vec<u32> },
    /// Move the first node of a connected piece (the others follow).
    SetStart { num: u32, p: [f64; 3] },
    /// Split an element at a fraction of its length with a new node.
    Split { id: u32, at: f64 },
    /// Number the nodes again in table order: start, start + step, …
    Renumber { start: u32, step: u32 },
    /// Turn a straight element into an expansion loop in its middle: out by `height` (sideways in
    /// the horizontal plane, or up when `up`), across by `width`, and back, with long-radius bends.
    Loop { id: u32, width: f64, height: f64, up: bool },
    /// Copy the elements `ids` (with their components and the supports on their nodes) and join the
    /// copy to node `at`, mirrored across the plane normal to axis 0, 1 or 2 (X, Y, Z) if given.
    Duplicate { ids: Vec<u32>, at: u32, mirror: Option<u8> },
    SetSettings { s: Settings, units: String },
    SetMeta { name: String, notes: String, project: Project },
    /// Replace the load cases (turns automatic cases off).
    SetCases { cases: Vec<LoadCase> },
    /// Replace the rotating equipment list.
    SetEquipment { list: Vec<crate::equip::Equipment> },
    /// Build the recommended load cases again (turns automatic cases on).
    RecommendCases,
    Batch(Vec<Op>),
}

#[derive(Clone, Debug, Default)]
pub struct Doc {
    pub m: Model,
    pub path: Option<String>,
    pub dirty: bool,
    undo: Vec<Model>,
    redo: Vec<Model>,
}

const UNDO_LIMIT: usize = 200;

impl Doc {
    pub fn new(m: Model) -> Doc {
        Doc { m, ..Default::default() }
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Apply an edit. Returns the ids it created.
    pub fn apply(&mut self, op: Op) -> Result<Vec<u32>, String> {
        let mut m = self.m.clone();
        let mut created = vec![];
        exec(&mut m, op, &mut created)?;
        finish(&mut m)?;
        if m == self.m {
            return Ok(created);
        }
        let old = std::mem::replace(&mut self.m, m);
        self.undo.push(old);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
        Ok(created)
    }

    pub fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(m) => {
                self.redo.push(std::mem::replace(&mut self.m, m));
                self.dirty = true;
                true
            }
            None => false,
        }
    }

    pub fn redo(&mut self) -> bool {
        match self.redo.pop() {
            Some(m) => {
                self.undo.push(std::mem::replace(&mut self.m, m));
                self.dirty = true;
                true
            }
            None => false,
        }
    }
}

/// After every edit: lay the nodes out, drop what hangs on vanished nodes, rebuild automatic cases,
/// and check every rule.
fn finish(m: &mut Model) -> Result<(), String> {
    m.layout()?;
    let nodes: HashSet<u32> = m.nodes.iter().map(|n| n.num).collect();
    m.supports.retain(|s| nodes.contains(&s.node));
    m.nozzles.retain(|s| nodes.contains(&s.node));
    m.loads.retain(|s| nodes.contains(&s.node));
    // Equipment keeps only nozzles that are still there.
    let zs: HashSet<u32> = m.nozzles.iter().map(|z| z.id).collect();
    for e in &mut m.equipment {
        e.nozzles.retain(|id| zs.contains(id));
    }
    if m.settings.auto_cases {
        crate::cases::rebuild(m);
    }
    m.next = m.next.max(m.max_id() + 1);
    m.check()
}

fn fresh(m: &mut Model) -> u32 {
    let id = m.next.max(m.max_id() + 1);
    m.next = id + 1;
    id
}

/// A node number not in use, near `want`.
pub fn free_num(m: &Model, want: u32) -> u32 {
    let used: HashSet<u32> = m.node_order().into_iter().collect();
    (want.max(1)..).find(|n| !used.contains(n)).unwrap_or(want)
}

fn exec(m: &mut Model, op: Op, created: &mut Vec<u32>) -> Result<(), String> {
    // Supports, nozzles and loads go on nodes that exist.
    let on = |m: &Model, n: u32| if m.node_order().contains(&n) { Ok(()) } else { Err(format!("There is no node {n} in the model.")) };
    match &op {
        Op::Add { item: Item::Support(x), .. } | Op::Set { item: Item::Support(x) } => on(m, x.node)?,
        Op::Add { item: Item::Nozzle(x), .. } | Op::Set { item: Item::Nozzle(x) } => on(m, x.node)?,
        Op::Add { item: Item::Load(x), .. } | Op::Set { item: Item::Load(x) } => on(m, x.node)?,
        _ => {}
    }
    match op {
        Op::Add { item, after } => {
            let id = fresh(m);
            created.push(id);
            match item {
                Item::Element(mut x) => {
                    x.id = id;
                    let at = after.and_then(|a| m.elements.iter().position(|e| e.id == a)).map(|i| i + 1).unwrap_or(m.elements.len());
                    m.elements.insert(at, x);
                }
                Item::Pipe(mut x) => {
                    x.id = id;
                    m.pipes.push(x);
                }
                Item::Material(mut x) => {
                    x.id = id;
                    m.materials.push(x);
                }
                Item::Cond(mut x) => {
                    x.id = id;
                    m.conds.push(x);
                }
                Item::Support(mut x) => {
                    x.id = id;
                    m.supports.push(x);
                }
                Item::Nozzle(mut x) => {
                    x.id = id;
                    // One nozzle per node.
                    m.nozzles.retain(|z| z.node != x.node);
                    m.nozzles.push(x);
                }
                Item::Load(mut x) => {
                    x.id = id;
                    m.loads.push(x);
                }
                Item::Case(mut x) => {
                    x.id = id;
                    m.cases.push(x);
                    m.settings.auto_cases = false;
                }
            }
        }
        Op::Set { item } => {
            fn put<T>(v: &mut [T], id: impl Fn(&T) -> u32, x: T) -> Result<(), String> {
                let want = id(&x);
                match v.iter_mut().find(|y| id(y) == want) {
                    Some(y) => {
                        *y = x;
                        Ok(())
                    }
                    None => Err("That item is no longer in the model.".into()),
                }
            }
            match item {
                Item::Element(x) => {
                    // Renaming a node in the table renames it in supports, nozzles and loads too,
                    // when no other element still uses the old number.
                    let old = m.elements.iter().find(|e| e.id == x.id).map(|e| (e.from, e.to));
                    let (nf, nt) = (x.from, x.to);
                    put(&mut m.elements, |e| e.id, x)?;
                    if let Some((of, ot)) = old {
                        for (o, n) in [(of, nf), (ot, nt)] {
                            if o != n && !m.elements.iter().any(|e| e.from == o || e.to == o) {
                                for s in &mut m.supports {
                                    if s.node == o {
                                        s.node = n;
                                    }
                                }
                                for s in &mut m.nozzles {
                                    if s.node == o {
                                        s.node = n;
                                    }
                                }
                                for s in &mut m.loads {
                                    if s.node == o {
                                        s.node = n;
                                    }
                                }
                                if let Some(nd) = m.nodes.iter_mut().find(|q| q.num == o) {
                                    nd.num = n;
                                }
                            }
                        }
                    }
                }
                Item::Pipe(x) => put(&mut m.pipes, |e| e.id, x)?,
                Item::Material(x) => put(&mut m.materials, |e| e.id, x)?,
                Item::Cond(x) => put(&mut m.conds, |e| e.id, x)?,
                Item::Support(x) => put(&mut m.supports, |e| e.id, x)?,
                Item::Nozzle(x) => put(&mut m.nozzles, |e| e.id, x)?,
                Item::Load(x) => put(&mut m.loads, |e| e.id, x)?,
                Item::Case(x) => {
                    put(&mut m.cases, |e| e.id, x)?;
                    m.settings.auto_cases = false;
                }
            }
        }
        Op::Delete { ids } => {
            let ids: HashSet<u32> = ids.into_iter().collect();
            let in_use = |id: u32, m: &Model| m.elements.iter().any(|e| !ids.contains(&e.id) && (e.pipe == id || e.material == id || e.cond == id || matches!(e.comp, crate::model::Comp::Reducer { to } if to == id)));
            for id in &ids {
                let prop = m.pipes.iter().any(|x| x.id == *id) || m.materials.iter().any(|x| x.id == *id) || m.conds.iter().any(|x| x.id == *id);
                if prop && in_use(*id, m) {
                    return Err("That pipe, material or set of line conditions is still used by an element.".into());
                }
            }
            if m.cases.iter().any(|c| ids.contains(&c.id)) {
                m.settings.auto_cases = false;
            }
            m.elements.retain(|x| !ids.contains(&x.id));
            m.pipes.retain(|x| !ids.contains(&x.id));
            m.materials.retain(|x| !ids.contains(&x.id));
            m.conds.retain(|x| !ids.contains(&x.id));
            m.supports.retain(|x| !ids.contains(&x.id));
            m.nozzles.retain(|x| !ids.contains(&x.id));
            m.loads.retain(|x| !ids.contains(&x.id));
            // A range that used a deleted case goes with it.
            let mut gone: HashSet<u32> = ids.clone();
            loop {
                let before: Vec<u32> = m.cases.iter().map(|c| c.id).collect();
                m.cases.retain(|c| !gone.contains(&c.id) && !matches!(c.recipe, crate::model::Recipe::Range { a, b } if gone.contains(&a) || gone.contains(&b)));
                if m.cases.len() == before.len() {
                    break;
                }
                let left: HashSet<u32> = m.cases.iter().map(|c| c.id).collect();
                gone.extend(before.into_iter().filter(|i| !left.contains(i)));
            }
        }
        Op::SetStart { num, p } => {
            if !p.iter().all(|v| v.is_finite()) {
                return Err("The coordinates must be numbers.".into());
            }
            let roots = roots(m);
            if !roots.contains(&num) {
                return Err(format!("Node {num} is placed by its elements: change their DX, DY, DZ, or move the first node of its run."));
            }
            if let Some(n) = m.nodes.iter_mut().find(|n| n.num == num) {
                n.x = p[0];
                n.y = p[1];
                n.z = p[2];
            }
        }
        Op::Split { id, at } => {
            if !(at > 0.0 && at < 1.0) {
                return Err("The new node must be inside the element.".into());
            }
            let i = m.elements.iter().position(|e| e.id == id).ok_or("That element is no longer in the model.")?;
            let e = m.elements[i].clone();
            let num = free_num(m, (e.from + (e.to.max(e.from) - e.to.min(e.from)) / 2).max(e.from.min(e.to) + 1));
            let a = Element { d: e.d.map(|v| v * at), to: num, comp: crate::model::Comp::Pipe, ..e.clone() };
            // The component at the To end stays with the second part.
            let b = Element { id: fresh(m), from: num, d: e.d.map(|v| v * (1.0 - at)), ..e };
            created.push(b.id);
            m.elements[i] = a;
            m.elements.insert(i + 1, b);
        }
        Op::Renumber { start, step } => {
            if start == 0 || step == 0 {
                return Err("Node numbers start at 1 and step by at least 1.".into());
            }
            let order = m.node_order();
            let map: HashMap<u32, u32> = order.iter().enumerate().map(|(i, n)| (*n, start + step * i as u32)).collect();
            for e in &mut m.elements {
                e.from = map[&e.from];
                e.to = map[&e.to];
            }
            for n in &mut m.nodes {
                n.num = map.get(&n.num).copied().unwrap_or(n.num);
            }
            for s in &mut m.supports {
                s.node = map.get(&s.node).copied().unwrap_or(s.node);
            }
            for s in &mut m.nozzles {
                s.node = map.get(&s.node).copied().unwrap_or(s.node);
            }
            for s in &mut m.loads {
                s.node = map.get(&s.node).copied().unwrap_or(s.node);
            }
            m.settings.step = step;
        }
        Op::Loop { id, width, height, up } => {
            let i = m.elements.iter().position(|e| e.id == id).ok_or("That element is no longer in the model.")?;
            let e = m.elements[i].clone();
            if !matches!(e.comp, crate::model::Comp::Pipe) {
                return Err("An expansion loop goes in a straight piece of pipe: choose an element with no component.".into());
            }
            let l = e.length();
            if !(width > 0.0 && height > 0.0 && width.is_finite() && height.is_finite()) {
                return Err("The loop needs a width and a height.".into());
            }
            let u = crate::mat::unit(e.d);
            let side = if up { [0.0, 1.0, 0.0] } else { crate::mat::unit(crate::mat::cross([0.0, 1.0, 0.0], u)) };
            if crate::mat::norm(side) < 0.5 || crate::mat::dot(side, u).abs() > 1e-6 {
                return Err("A loop sideways needs a horizontal pipe; a loop upwards needs a pipe that is not vertical.".into());
            }
            let rb = crate::geom::bend_radius(m, crate::model::BendKind::Long, 0.0, e.pipe);
            if width < 2.0 * rb + 1e-6 || height < 2.0 * rb + 1e-6 || l < width + 2.0 * rb + 1e-6 {
                return Err(format!("The loop does not fit: with {:.0} mm bends it needs a width and height of at least {:.0} mm, and an element longer than the width by {:.0} mm.", rb * 1000.0, 2000.0 * rb, 2000.0 * rb));
            }
            let a = (l - width) / 2.0;
            let step = m.settings.step.max(1);
            let mut nums = vec![];
            let mut want = e.from.min(e.to) + 1;
            for _ in 0..4 {
                let n = free_num(m, want);
                // Keep the four new numbers apart from each other too.
                let n = (n..).find(|x| !nums.contains(x) && !m.node_order().contains(x)).unwrap_or(n);
                nums.push(n);
                want = n + 1;
            }
            let _ = step;
            let legs: [(f64, [f64; 3]); 5] = [(a, u), (height, side), (width, u), (height, crate::mat::scale(side, -1.0)), (a, u)];
            let ends = [e.from, nums[0], nums[1], nums[2], nums[3], e.to];
            let mut new = vec![];
            for k in 0..5 {
                let d = crate::mat::scale(legs[k].1, legs[k].0);
                let comp = if k < 4 { crate::model::Comp::Bend { bend: crate::model::BendKind::Long, r: 0.0 } } else { crate::model::Comp::Pipe };
                let id = if k == 0 { e.id } else { fresh(m) };
                if k > 0 {
                    created.push(id);
                }
                new.push(Element { id, from: ends[k], to: ends[k + 1], d, comp, ..e.clone() });
            }
            m.elements.splice(i..=i, new);
        }
        Op::Duplicate { ids, at, mirror } => {
            if !m.node_order().contains(&at) {
                return Err(format!("There is no node {at} to join the copy to."));
            }
            let picked: Vec<Element> = m.elements.iter().filter(|e| ids.contains(&e.id)).cloned().collect();
            if picked.is_empty() {
                return Err("Choose the elements to copy.".into());
            }
            if mirror.is_some_and(|a| a > 2) {
                return Err("Mirror across X, Y or Z.".into());
            }
            // The section's first node joins `at`; the others get new numbers after the largest.
            let first = picked[0].from;
            let step = m.settings.step.max(1);
            let mut next = m.node_order().into_iter().max().unwrap_or(0);
            let mut map: HashMap<u32, u32> = HashMap::new();
            map.insert(first, at);
            let mut out = vec![];
            for e in &picked {
                for n in [e.from, e.to] {
                    map.entry(n).or_insert_with(|| {
                        next = (next / step + 1) * step;
                        next
                    });
                }
                let mut d = e.d;
                if let Some(ax) = mirror {
                    d[ax as usize] = -d[ax as usize];
                }
                let id = fresh(m);
                created.push(id);
                out.push(Element { id, from: map[&e.from], to: map[&e.to], d, ..e.clone() });
            }
            m.elements.extend(out);
            let sups: Vec<Support> = m.supports.iter().filter(|s| s.node != first && map.contains_key(&s.node)).cloned().collect();
            for mut s in sups {
                s.id = fresh(m);
                s.node = map[&s.node];
                if let (Some(ax), crate::model::SupKind::Restraint { dir }) = (mirror, &mut s.kind) {
                    dir[ax as usize] = -dir[ax as usize];
                }
                created.push(s.id);
                m.supports.push(s);
            }
        }
        Op::SetSettings { s, units } => {
            if units != "SI" && units != "US" {
                return Err("Units are SI or US.".into());
            }
            m.settings = s;
            m.units = units;
        }
        Op::SetMeta { name, notes, project } => {
            m.name = if name.trim().is_empty() { "Untitled".into() } else { name };
            m.notes = notes;
            m.project = project;
        }
        Op::SetEquipment { list } => {
            m.equipment = list;
        }
        Op::SetCases { cases } => {
            m.cases = cases;
            m.settings.auto_cases = false;
            // New cases may come with id 0.
            let mut next = m.next.max(m.max_id() + 1);
            for c in &mut m.cases {
                if c.id == 0 {
                    c.id = next;
                    next += 1;
                }
            }
            m.next = next;
        }
        Op::RecommendCases => {
            m.settings.auto_cases = true;
        }
        Op::Batch(ops) => {
            for o in ops {
                exec(m, o, created)?;
            }
        }
    }
    Ok(())
}

/// The first node of each connected piece: the nodes whose coordinates are kept.
pub fn roots(m: &Model) -> Vec<u32> {
    let mut parent: HashMap<u32, u32> = HashMap::new();
    fn find(p: &mut HashMap<u32, u32>, x: u32) -> u32 {
        let mut r = x;
        while let Some(&q) = p.get(&r) {
            if q == r {
                break;
            }
            r = q;
        }
        p.insert(x, r);
        r
    }
    let order = m.node_order();
    for n in &order {
        parent.insert(*n, *n);
    }
    for e in &m.elements {
        let (a, b) = (find(&mut parent, e.from), find(&mut parent, e.to));
        if a != b {
            // Keep the earlier node as the root.
            let (ia, ib) = (order.iter().position(|x| *x == a), order.iter().position(|x| *x == b));
            if ia < ib {
                parent.insert(b, a);
            } else {
                parent.insert(a, b);
            }
        }
    }
    order.iter().copied().filter(|n| find(&mut parent, *n) == *n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el(from: u32, to: u32, d: [f64; 3]) -> Item {
        Item::Element(Element { from, to, d, pipe: 1, material: 2, cond: 3, ..Default::default() })
    }

    #[test]
    fn a_table_lays_out_and_edits_undo() {
        let mut d = Doc::new(Model::new());
        d.apply(Op::Add { item: el(10, 20, [3.0, 0.0, 0.0]), after: None }).unwrap();
        d.apply(Op::Add { item: el(20, 30, [0.0, 0.0, -2.0]), after: None }).unwrap();
        let ids = d.apply(Op::Add { item: el(30, 40, [0.0, 4.0, 0.0]), after: None }).unwrap();
        let p = d.m.node(40).unwrap().p();
        assert_eq!(p, [3.0, 4.0, -2.0]);
        // A loop that does not close is refused and changes nothing.
        let before = d.m.clone();
        assert!(d.apply(Op::Add { item: el(40, 10, [0.0, 0.0, 1.0]), after: None }).is_err());
        assert_eq!(before, d.m);
        // Moving the start moves everything.
        d.apply(Op::SetStart { num: 10, p: [1.0, 1.0, 1.0] }).unwrap();
        assert_eq!(d.m.node(40).unwrap().p(), [4.0, 5.0, -1.0]);
        assert!(d.apply(Op::SetStart { num: 30, p: [0.0; 3] }).is_err());
        // Split, renumber, undo.
        d.apply(Op::Split { id: ids[0], at: 0.25 }).unwrap();
        assert_eq!(d.m.elements.len(), 4);
        let mid = d.m.elements[2].to;
        assert!((d.m.node(mid).unwrap().y - 2.0).abs() < 1e-12, "{:?}", d.m.nodes);
        d.apply(Op::Renumber { start: 100, step: 5 }).unwrap();
        assert_eq!(d.m.node_order(), vec![100, 105, 110, 115, 120]);
        assert!(d.undo() && d.undo());
        assert_eq!(d.m.node_order(), vec![10, 20, 30, 40]);
        // An expansion loop in the 3 m element 10–20: five elements, four bends, same end point.
        let mut d2 = Doc::new(Model::new());
        d2.apply(Op::Add { item: el(10, 20, [12.0, 0.0, 0.0]), after: None }).unwrap();
        let id = d2.m.elements[0].id;
        assert!(d2.apply(Op::Loop { id, width: 0.2, height: 3.0, up: false }).is_err());
        d2.apply(Op::Loop { id, width: 3.0, height: 2.5, up: false }).unwrap();
        assert_eq!(d2.m.elements.len(), 5);
        assert_eq!(d2.m.node(20).unwrap().p(), [12.0, 0.0, 0.0]);
        assert!(d2.m.nodes.iter().any(|n| (n.z.abs() - 2.5).abs() < 1e-12));
        assert!(crate::geom::Geom::build(&d2.m).unwrap().warnings.is_empty());
        // Copy it mirrored across X onto node 20: the copy runs back west.
        let ids: Vec<u32> = d2.m.elements.iter().map(|e| e.id).collect();
        d2.apply(Op::Duplicate { ids, at: 20, mirror: Some(0) }).unwrap();
        assert_eq!(d2.m.elements.len(), 10);
        let last = d2.m.elements.last().unwrap().to;
        assert!(d2.m.node(last).unwrap().x.abs() < 1e-9, "{:?}", d2.m.node(last));
        // Supports go with their node.
        d.apply(Op::Add { item: Item::Support(Support { node: 40, ..Default::default() }), after: None }).unwrap();
        let last = d.m.elements.last().unwrap().id;
        d.apply(Op::Delete { ids: vec![last] }).unwrap();
        assert!(d.m.supports.is_empty() && d.m.node(40).is_none());
        // A pipe in use cannot be deleted.
        assert!(d.apply(Op::Delete { ids: vec![1] }).is_err());
    }
}
