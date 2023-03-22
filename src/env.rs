use egg::{
    Analysis, AstSize, EGraph, Extractor, Id, Language, RecExpr, Rewrite, Runner, SimpleScheduler,
    StopReason,
};
use std::collections::HashMap;
use std::time::Duration;

pub struct Env<L: Language, N: Analysis<L>> {
    action_history: Vec<usize>,
    expr: RecExpr<L>,
    egraph: EGraph<L, N>,
    root_id: Id,
    num_rules: usize,
    rules: Vec<Rewrite<L, N>>,

    node_limit: usize,
    time_limit: std::time::Duration,

    base_cost: usize,
    cnt: u32,
    sat_counter: usize,
}

impl<L: Language + egg::FromOp, N: Analysis<L> + Clone + std::default::Default> Env<L, N> {
    pub fn new(
        expr: &'static str,
        rules: Vec<Rewrite<L, N>>,
        node_limit: usize,
        time_limit: usize,
    ) -> Self {
        // get base
        let e = expr.clone().parse().unwrap();
        let runner: Runner<L, N> = Runner::default().with_expr(&e);
        let (base_cost, _) = Extractor::new(&runner.egraph, AstSize).find_best(runner.roots[0]);
        Env {
            action_history: Vec::new(),
            expr: e,
            egraph: EGraph::default(),
            root_id: Id::default(),
            num_rules: rules.len(),
            rules: rules,
            node_limit: node_limit,
            time_limit: Duration::from_secs(time_limit.try_into().unwrap()),

            base_cost: base_cost,
            cnt: 0,
            sat_counter: 0,
        }
    }
    pub fn reset(&mut self) {
        self.action_history.clear();
        self.cnt = 0;
        self.sat_counter = 0;
        self.egraph = EGraph::default();
        self.root_id = self.egraph.add_expr(&self.expr);
    }

    pub fn step(&mut self, action: usize) -> ((), f32, bool, HashMap<u32, u32>) {
        // run egg
        let egraph = std::mem::take(&mut self.egraph);
        let rule = vec![self.rules[action].clone()];
        let mut runner: Runner<L, N> = Runner::default()
            .with_egraph(egraph)
            .with_iter_limit(1)
            .with_node_limit(self.node_limit)
            .with_time_limit(self.time_limit)
            .with_scheduler(SimpleScheduler)
            .run(&rule);
        let num_applications: usize = runner
            .iterations
            .iter()
            .map(|i| i.applied.values().sum::<usize>())
            .sum();
        let egraph_nodes: usize = runner.egraph.total_size();
        let egraph_classes: usize = runner.egraph.number_of_classes();

        // run extract
        runner.egraph.rebuild(); // invariant
        let extractor = Extractor::new(&runner.egraph, AstSize);
        let (best_cost, _) = extractor.find_best(self.root_id);

        // reclaim the partial egraph
        self.egraph = runner.egraph;

        // compute transition
        self.cnt += 1;
        let mut done = false;
        match runner.stop_reason.unwrap() {
            StopReason::NodeLimit(_) | StopReason::TimeLimit(_) => {
                done = true;
                self.sat_counter = 0;
            }
            StopReason::Saturated => {
                self.sat_counter += 1;
                if self.sat_counter == (self.num_rules) {
                    done = true;
                }
            }
            StopReason::IterationLimit(_) => self.sat_counter = 0,
            _ => self.sat_counter = 0,
        }
        let reward = std::cmp::max(self.base_cost - best_cost, 0); // TODO allow callback cost func
        self.action_history.push(action);

        ((), (reward as f32), done, HashMap::new())
    }

    pub fn get_action_space(&self) -> usize {
        self.num_rules
    }

    pub fn checkpoint(&self) -> Vec<usize> {
        self.action_history.clone()
    }

    pub fn restore(&mut self, checkpoint_data: Vec<usize>) {}
}

#[cfg(test)]
mod test {
    #![allow(unused_imports)]
    use super::*;
    use egg::*;
    use std::thread::sleep;
    use std::time::Duration;

    define_language! {
        enum SimpleLanguage {
            Num(i32),
            "+" = Add([Id; 2]),
            "*" = Mul([Id; 2]),
            Symbol(Symbol),
        }
    }

    fn make_rules() -> Vec<Rewrite<SimpleLanguage, ()>> {
        vec![
            rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
            rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
            rewrite!("add-0"; "(+ ?a 0)" => "?a"),
            rewrite!("mul-0"; "(* ?a 0)" => "0"),
            rewrite!("mul-1"; "(* ?a 1)" => "?a"),
        ]
    }

    const NODE_LIMIT: usize = 10000;
    const TIME_LIMIT: usize = 10;

    #[test]
    fn test_add_expr_all_return_the_all_id() {
        // TODO
    }
}
