use std::collections::{HashSet, HashMap, VecDeque};
use std::ops::{Index, IndexMut};
use std::fmt;

use crate::tree::Tree;
use crate::strategy::Strategy;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum AgentKind {
    Application,
    Bracket,
    Croissant,
    Duplicator,
    Eraser,
    Lambda,
    Root,
}

fn order(a : AgentKind, b : AgentKind) -> (AgentKind, AgentKind) {
    (a.min(b), a.max(b))
}

#[derive(Eq, Hash, Clone)]
pub struct Wire {
    source : usize,
    target : usize
}

impl PartialEq for Wire {
    fn eq(&self, other : &Wire) -> bool {
        let (s1, t1) = (self.source, self.target);
        let (s2, t2) = (other.source, other.target);
        (s1 == s2 && t1 == t2) || (s1 == t2 && t1 == s2)
    }
}

impl fmt::Debug for Wire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.target, self.source)
    }
}

impl Wire {
    pub fn new(source : usize, target : usize) -> Wire {
        Wire { source, target }
    }

    pub fn fill(&mut self, id : usize) {
        if self.target == 0 {
            self.target = id;
        } else {
            self.source = id;
        }
    }

    pub fn swap(&mut self) {
        let temp = self.source;
        self.source = self.target;
        self.target = temp;
    }
}

#[derive(Clone)]
pub struct Agent {
    kind : AgentKind,
    level : isize,
    wires : [usize; 3]
}

impl Agent {
    fn new(kind : AgentKind, level : isize, wires : Vec<usize>) -> Agent {
        let mut result = Agent {
            kind,
            level,
            wires: [0, 0, 0]
        };
        result.update(wires);
        result
    }

    fn len(&self) -> usize {
        match self.kind {
            | AgentKind::Application
            | AgentKind::Lambda
            | AgentKind::Duplicator
            => 3,
            | AgentKind::Eraser
            | AgentKind::Root
            => 1,
            _ => 2
        }
    }

    fn update(&mut self, wires : Vec<usize>) {
        for i in 0..self.len() {
            self[i] = wires[i];
        }
    }

    fn port_of(&self, wire : usize) -> usize {
        let mut result = 0;
        for i in 0..self.len() {
            if self.wires[i] == wire {
                result = i;
            }
        }
        result
    }
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header = match self.kind {
            AgentKind::Root => "R",
            AgentKind::Lambda => "L",
            AgentKind::Croissant => "C",
            AgentKind::Bracket => "B",
            AgentKind::Application => "@",
            AgentKind::Duplicator => "D",
            AgentKind::Eraser => "e"
        };

        match self.kind {
            AgentKind::Root | AgentKind::Eraser
                => write!(f, "{}[{}]", header, self[0]),
            AgentKind::Bracket | AgentKind::Croissant
                => write!(f, "{}[{}, {}]", header, self[0], self[1]),
            _ => write!(f, "{}[{},{},{}]", header, self[0], self[1], self[2]),
        }
    }
}

impl Index<usize> for Agent {
    type Output = usize;

    fn index(&self, index : usize) -> &usize {
        &self.wires[index]
    }
}

impl IndexMut<usize> for Agent {
    fn index_mut(&mut self, index: usize) -> &mut usize {
        &mut self.wires[index]
    }
}

#[derive(Debug)]
pub struct Net {
    agent_id : usize,
    wire_id : usize,
    agents : HashMap<usize, Agent>,
    wires : HashMap<usize, Wire>
}

impl Net {
    pub fn new() -> Net {
        Net {
            agent_id: 1,
            wire_id: 1,
            agents: HashMap::new(),
            wires: HashMap::new()
        }
    }

    pub fn partner(&self, agent_id : usize, wire_id : usize) -> usize {
        let wire = self.wire(wire_id);
        if agent_id == wire.source {
            wire.target
        } else {
            wire.source
        }
    }

    pub fn to_tree(&self) -> Option<Tree> {
        let mut map = HashMap::new();
        let result = self.to_tree_helper(1, self.agent(1)[0], &mut map, 1000);
        result.map(|x| Tree::fix_indices(x))
    }

    pub fn to_tree_helper(&self, aid : usize, wid : usize, oracle : &mut HashMap<usize, usize>, gas : isize) -> Option<Tree> {
        use AgentKind::*;
        if gas <= 0 { return None; }
        let agent = self.agent(aid);
        match agent.kind {
            Root => {
                let next_id = self.partner(aid, agent[0]);
                self.to_tree_helper(next_id, agent[0], oracle, gas)
            },
            Bracket => {
                let next_id = self.partner(aid, agent[1]);
                self.to_tree_helper(next_id, agent[1], oracle, gas)
            },
            Croissant => {
                let next_id = self.partner(aid, agent[1]);
                self.to_tree_helper(next_id, agent[1], oracle, gas)
            },
            Application => {
                let left_id = self.partner(aid, agent[0]);
                let right_id = self.partner(aid, agent[2]);
                let left = self.to_tree_helper(left_id, agent[0], oracle, gas - 1);
                let right = self.to_tree_helper(right_id, agent[2], oracle, gas - 1);
                if let Some(le) = left {
                    if let Some(ri) = right {
                        Some(Tree::App(
                            Box::new(le),
                            Box::new(ri)
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            Lambda => {
                let port = self.agent(aid).port_of(wid);
                if port == 0 {
                    let body_id = self.partner(aid, agent[1]);
                    let body = self.to_tree_helper(body_id, agent[1], oracle, gas - 1);
                    if let Some(b) = body {
                        Some(Tree::Abs(aid, Box::new(b)))
                    } else {
                        None
                    }
                } else {
                    Some(Tree::Var(aid))
                }
            },
            Duplicator => {
                let port = self.agent(aid).port_of(wid);
                if port == 0 {
                    if let Some(&p) = oracle.get(&aid) {
                        let body_id = self.partner(aid, agent[p]);
                        self.to_tree_helper(body_id, agent[p], oracle, gas - 1)
                    } else {
                        None
                    }
                } else {
                    let body_id = self.partner(aid, agent[0]);
                    oracle.insert(aid, port);
                    self.to_tree_helper(body_id, agent[0], oracle, gas - 1)
                }
            },
            Eraser => {
                None
            }
        }
    }

    pub fn from_tree(tree : &Tree) -> Net {
        let mut net = Net::new();
        let mut map = HashMap::new();
        let root_id = net.add_agent(Agent::new(AgentKind::Root, 0, vec![0]));
        let root_wire = net.add_wire(Wire::new(root_id, 0));
        net.mut_agent(root_id)[0] = root_wire;
        let remaining = net.from_tree_helper(tree, root_wire, 0, &mut map);
        net.mut_wire(root_wire).target = remaining;
        net.fix_dangling_lambdas();
        net
    }

    fn fix_dangling_lambdas(&mut self) {
        let mut agents = vec![];
        for (key, val) in self.agents.iter() {
            if val[2] == 0 && val.kind == AgentKind::Lambda {
                agents.push(*key);
            }
        }

        for id in agents {
            let eid = self.add_agent(Agent::new(AgentKind::Eraser, 0, vec![0]));
            let wire = self.add_wire(Wire::new(id, eid));
            self.mut_agent(eid)[0] = wire;
            self.mut_agent(id)[2] = wire;
        }
    }

    fn from_tree_helper(&mut self, tree : &Tree, dangling : usize, level : isize, name_map : &mut HashMap<usize, usize>) -> usize {
        match tree {
            Tree::Var(id_isize) => {
                let id = *id_isize as usize;
                let lambda_id = *name_map.get(&id).expect("Free variables are not supported.");

                let croissant_wire = self.add_wire(Wire::new(0, 0));
                let croissant = self.add_agent(Agent::new(AgentKind::Croissant, level, vec![croissant_wire, dangling]));
                self.mut_wire(croissant_wire).source = croissant;
                let dangling = if level > 0 {
                    let bracket_wire = self.add_wire(Wire::new(0, 0));
                    let bracket = self.add_agent(Agent::new(AgentKind::Bracket, level - 1, vec![bracket_wire, croissant_wire]));
                    self.mut_wire(bracket_wire).source = bracket;

                    self.mut_wire(croissant_wire).target = bracket;
                    let mut previous_wire = bracket_wire;
                    for i in (0..(level - 1)).rev() {
                        let temp_wire = self.add_wire(Wire::new(0, 0));
                        let temp = self.add_agent(Agent::new(AgentKind::Bracket, i, vec![temp_wire, previous_wire]));
                        self.mut_wire(temp_wire).source = temp;

                        self.mut_wire(previous_wire).target = temp;
                        previous_wire = temp_wire;
                    }
                    previous_wire
                } else {
                    croissant_wire
                };

                if self.agent(lambda_id)[2] == 0 {
                    self.mut_agent(lambda_id)[2] = dangling;
                    self.mut_wire(dangling).target = lambda_id;
                } else {
                    let previous_wire = self.agent(lambda_id)[2];
                    let new_wire = self.add_wire(Wire::new(lambda_id, 0));
                    let dup_id = self.add_agent(Agent::new(
                        AgentKind::Duplicator,
                        level,
                        vec![new_wire, previous_wire, dangling])
                    );
                    self.mut_wire(new_wire).fill(dup_id);
                    self.mut_agent(lambda_id)[2] = new_wire;
                    if self.wire(previous_wire).target == lambda_id {
                        self.mut_wire(previous_wire).target = dup_id;
                    } else {
                        self.mut_wire(previous_wire).source = dup_id;
                    }
                    self.mut_wire(dangling).target = dup_id;
                }
                croissant
            },
            Tree::Abs(id_isize, body) => {
                let id = *id_isize as usize;

                let lambda_id = self.add_agent(Agent::new(AgentKind::Lambda, level, vec![0, 0, 0]));
                let body_wire = self.add_wire(Wire::new(lambda_id, 0));
                self.mut_agent(lambda_id).update(vec![dangling, body_wire, 0]);

                name_map.insert(id, lambda_id);
                let body_id = self.from_tree_helper(body, body_wire, level, name_map);
                self.mut_wire(body_wire).fill(body_id);
                lambda_id
            },
            Tree::App(left, right) => {
                let application_id = self.add_agent(Agent::new(AgentKind::Application, level, vec![0, 0, 0]));
                let left_wire = self.add_wire(Wire::new(application_id, 0));
                let right_wire = self.add_wire(Wire::new(application_id, 0));
                self.mut_agent(application_id).update(vec![left_wire, dangling, right_wire]);

                let left_id = self.from_tree_helper(left, left_wire, level, name_map);
                self.mut_wire(left_wire).fill(left_id);

                let right_id = self.from_tree_helper(right, right_wire, level + 1, name_map);
                self.mut_wire(right_wire).fill(right_id);
                application_id
            }
        }
    }

    fn add_agent(&mut self, agent : Agent) -> usize {
        self.agents.insert(self.agent_id, agent);
        self.agent_id += 1;
        self.agent_id - 1
    }

    fn mut_agent(&mut self, id : usize) -> &mut Agent {
        self.agents.get_mut(&id)
            .expect(format!("fn mut_agent {} failed", id).as_str())
    }

    fn agent(&self, id : usize) -> &Agent {
        self.agents.get(&id)
            .expect(format!("fn agent {} failed", id).as_str())
    }

    fn add_wire(&mut self, wire : Wire) -> usize {
        self.wires.insert(self.wire_id, wire);
        self.wire_id += 1;
        self.wire_id - 1
    }

    fn mut_wire(&mut self, id : usize) -> &mut Wire {
        self.wires.get_mut(&id)
            .expect(format!("fn mut_wire {} failed", id).as_str())
    }

    fn wire(&self, id : usize) -> &Wire {
        self.wires.get(&id)
            .expect(format!("fn wire {} failed", id).as_str())
    }

    pub fn replace(&mut self,
        port : usize,
        wire_id : usize,
        old_id : usize,
        new_id : usize)
    {
        let wire = self.wires.get_mut(&wire_id)
            .expect(format!("fn replace, wire_id {}, missing", wire_id).as_str());
        let new = self.agents.get_mut(&new_id)
            .expect(format!("fn replace, agent_id {}, missing", new_id).as_str());
        if wire.source == old_id {
            wire.source = new_id;
            new[port] = wire_id;
        } else {
            wire.target = new_id;
            new[port] = wire_id;
        }
    }

    pub fn connect(&mut self,
        dangling1_id : usize,
        wire1_id : usize,
        dangling2_id : usize,
        wire2_id : usize)
    {
        let wire1 = self.wires.remove(&wire1_id)
            .expect(format!("fn connect, wire_id {}, missing", wire1_id).as_str());
        let wire2 = self.wires.remove(&wire2_id)
            .expect(format!("fn connect, wire_id {}, missing", wire2_id).as_str());
        let agent1_id = if wire1.source == dangling1_id
            { wire1.target }
            else { wire1.source };
        let agent2_id = if wire2.source == dangling2_id
            { wire2.target }
            else { wire2.source };
        let port1 = self.agent(agent1_id).port_of(wire1_id);
        let port2 = self.agent(agent2_id).port_of(wire2_id);
        let wire = Wire::new(agent1_id, agent2_id);
        let id = self.add_wire(wire);
        self.mut_agent(agent1_id)[port1] = id;
        self.mut_agent(agent2_id)[port2] = id;
    }

    fn find_beta_pairs(&self) -> HashSet<usize> {
        use AgentKind::*;
        let mut set = HashSet::new();
        for (_, agent) in self.agents.iter() {
            let incident = agent[0];
            let wire = self.wire(incident);
            let pair = order(self.agent(wire.source).kind,
                self.agent(wire.target).kind);
            match pair {
                (Application, Lambda) => {
                    set.insert(incident);
                },
                _ => { }
            }
        }
        set
    }

    pub fn reduction_step(&mut self, id : usize) -> bool {
        use AgentKind::*;

        let mut result = true;
        let (incident, wid) = {
            let agent = self.agents.get(&id);
            if agent.is_none() { return false; }
            let agent = agent.expect("impossible");
            let wire = self.wires.remove(&agent[0]);
            if wire.is_none() { return false; }
            let wire = wire.expect("impossible");
            (wire, agent[0])
        };
        let (agent, aid, partner, pid) = {
            let source = self.agents.remove(&incident.source);
            let target = self.agents.remove(&incident.target);
            if source.is_none() || target.is_none() { return false; }
            let source = source.expect("impossible");
            let target = target.expect("impossible");
            if source.kind <= target.kind {
                (source, incident.source, target, incident.target)
            } else {
                (target, incident.target, source, incident.source)
            }
        };
        let kinds = (agent.kind, partner.kind);
        let level = agent.level == partner.level;
        
        match (kinds.0, kinds.1, level) {
            (_, Eraser, true) | (Eraser, _, true) => {
                // Determine who is erasing who
                let (_eraser, _eid, partner, pid) = {
                    if agent.kind == AgentKind::Eraser {
                        (agent, aid, partner, pid)
                    } else {
                        (partner, pid, agent, aid)
                    }
                };

                if partner.len() > 1 {
                    let eraser_one = Agent::new(AgentKind::Eraser, 0, vec![0]);
                    let eraser_two = eraser_one.clone();
                    let e1id = self.add_agent(eraser_one);
                    let e2id = self.add_agent(eraser_two);
                    self.replace(0, partner[1], pid, e1id);
                    self.replace(0, partner[2], pid, e2id);
                }
            },
            (Application, Lambda, true) | (Duplicator, Duplicator, true) => {
                if agent[1] == agent[2] {
                    self.wires.remove(&agent[1]);
                    self.connect(pid, partner[1], pid, partner[2]);
                } else if partner[1] == partner[2] {
                    self.wires.remove(&partner[1]);
                    self.connect(aid, agent[1], aid, agent[2]);
                } else if agent[1] == partner[1] {
                    self.wires.remove(&agent[1]);
                    self.connect(aid, agent[2], pid, partner[2]);
                } else if agent[2] == partner[2] {
                    self.wires.remove(&agent[2]);
                    self.connect(aid, agent[1], pid, partner[1]);
                } else {
                    self.connect(aid, agent[1], pid, partner[1]);
                    self.connect(aid, agent[2], pid, partner[2]);
                }
            },
            (Croissant, Croissant, true) | (Bracket, Bracket, true) => {
                self.connect(aid, agent[1], pid, partner[1]);
            }
            (Duplicator, Lambda, false)
            | (Application, Duplicator, false)
            | (Duplicator, Duplicator, false) => {
                let agent1_id = self.add_agent(agent.clone());
                let agent2_id = self.add_agent(agent.clone());
                let partner1_id = self.add_agent(partner.clone());
                let partner2_id = self.add_agent(partner.clone());

                let wire1x2 = self.add_wire(Wire::new(agent1_id, partner2_id));
                let wire2x1 = self.add_wire(Wire::new(agent2_id, partner1_id));
                let wire1x1 = self.add_wire(Wire::new(agent1_id, partner1_id));
                let wire2x2 = self.add_wire(Wire::new(agent2_id, partner2_id));

                self.mut_agent(agent1_id).update(vec![partner[1], wire1x1, wire1x2]);
                self.mut_agent(agent2_id).update(vec![partner[2], wire2x1, wire2x2]);
                self.mut_agent(partner1_id).update(vec![agent[1], wire1x1, wire2x1]);
                self.mut_agent(partner2_id).update(vec![agent[2], wire1x2, wire2x2]);

                self.replace(0, partner[1], pid, agent1_id);
                self.replace(0, partner[2], pid, agent2_id);
                self.replace(0, agent[1], aid, partner1_id);
                self.replace(0, agent[2], aid, partner2_id);
            },
            (Croissant, Lambda, false)
            | (Application, Croissant, false)
            | (Croissant, Duplicator, false)
            | (Bracket, Lambda, false)
            | (Application, Bracket, false)
            | (Bracket, Duplicator, false) => {
                let akind = agent.kind;
                let (control, _cid, partner, pid) =
                    if akind == Croissant || akind == Bracket {
                        (agent, aid, partner, pid)
                    } else {
                        (partner, pid, agent, aid)
                    };
                let dlvl = if akind == Croissant {
                        -1
                    } else {
                        1
                    };
                self.agents.insert(pid, partner.clone());
                let control1_id = self.add_agent(control.clone());
                let control2_id = self.add_agent(control.clone());
                let old = vec![control[1], partner[1], partner[2]];
                self.mut_agent(pid).update(vec![old[0], control1_id, control2_id]);
                self.mut_agent(pid).level += dlvl;
                self.mut_agent(control1_id).update(vec![partner[1], pid]);
                self.mut_agent(control2_id).update(vec![partner[2], pid]);
            },
            _ => {
                // Undo what has been done
                self.wires.insert(wid, incident);
                self.agents.insert(aid, agent);
                self.agents.insert(pid, partner);
                result = false;
            }
        }
        result
    }

    fn reduce(&mut self) {
        let mut betas = self.find_beta_pairs();
        let mut queue : VecDeque<_> = betas.drain().collect();
        while let Some(id) = queue.pop_front() {
            let agent = self.agent(id);
            let mut branches = vec![];
            for i in 0..agent.len() {
                if i != 0 {
                    let pid = self.partner(id, agent[i]);
                    branches.push(pid);
                }
            }
            if self.reduction_step(id) {
                queue.extend(branches.iter());
            }
        }
    }
}

impl Strategy for Net {
    fn build(&mut self, tree : &Tree) {
        *self = Net::from_tree(tree);
    }

    fn reduce(&mut self) -> Option<u64> {
        self.reduce();
        let tree = self.to_tree();
        tree.and_then(|x| x.convert())
    }

    fn name(&self) -> String {
        String::from("optimal")
    }
}
