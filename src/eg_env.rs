use egg::{
    Analysis, AstSize, EGraph, Extractor, Id, Language, RecExpr, Report, Rewrite, Runner,
    SimpleScheduler, StopReason,
};
// use std::collections::HashMap;
use std::time::Duration;
use crate::env::Info;

pub struct EG_Env<L: Language, N: Analysis<L>> {
    expr: RecExpr<L>,
    egraph: EGraph<L, N>,
    root_id: Id,
    num_rules: usize,
    rules: Vec<Rewrite<L, N>>,

    node_limit: usize,
    time_limit: std::time::Duration,

    pub base_cost: usize,
    pub last_cost: usize,
    cnt: u32,
    sat_counter: usize,
}

impl<
        L: Language + egg::FromOp + std::marker::Send,
        N: Analysis<L> + Clone + std::default::Default,
    > EG_Env<L, N>
{
    pub fn new(
        expr: RecExpr<L>,
        rules: Vec<Rewrite<L, N>>,
        node_limit: usize,
        time_limit: usize,
    ) -> Self {
        // get base
        let runner: Runner<L, N> = Runner::default().with_expr(&expr);
        let (base_cost, _) = Extractor::new(&runner.egraph, AstSize).find_best(runner.roots[0]);
        EG_Env {
            expr: expr,
            egraph: EGraph::default(),
            root_id: Id::default(),
            num_rules: rules.len(),
            rules: rules,
            node_limit: node_limit,
            time_limit: Duration::from_secs(time_limit.try_into().unwrap()),

            base_cost: base_cost,
            last_cost: base_cost,
            cnt: 0,
            sat_counter: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cnt = 0;
        self.sat_counter = 0;
        self.egraph = EGraph::default();
        self.root_id = self.egraph.add_expr(&self.expr);
        self.last_cost = self.base_cost;
    }

    pub fn step(&mut self, action: usize) -> ((), f32, bool, Info) {
        // run egg
        let egraph = std::mem::take(&mut self.egraph);
        let rule = vec![self.rules[action].clone()];
        let runner: Runner<L, N> = Runner::default()
            .with_egraph(egraph)
            .with_iter_limit(1)
            .with_node_limit(self.node_limit)
            .with_time_limit(self.time_limit)
            .with_scheduler(SimpleScheduler)
            .run(&rule);
        let report = runner.report();

        // reclaim the partial egraph
        self.egraph = runner.egraph;

        // let num_applications: usize = runner
        //     .iterations
        //     .iter()
        //     .map(|i| i.applied.values().sum::<usize>())
        //     .sum();

        // run extract
        let extractor = Extractor::new(&self.egraph, AstSize);
        let (best_cost, _) = extractor.find_best(self.root_id);

        // compute transition
        self.cnt += 1;
        let mut done = false;
        match runner.stop_reason.as_ref().unwrap() {
            StopReason::NodeLimit(_) => {
                done = true;
                self.sat_counter = 0;
            }
            StopReason::TimeLimit(time) => {
                // TODO think about how this enables dealing with straggelers!
                panic!("egg time limit {}", time);
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
        let reward = std::cmp::max(self.last_cost - best_cost, 0); // TODO allow callback cost func
        self.last_cost = best_cost;
        let info = Info {
            report: report,
            best_cost: best_cost,
        };

        ((), (reward as f32), done, info)
    }

    // immediately extract and get reward
    // pub fn get_reward(&self) -> f32 {
    //     let extractor = Extractor::new(&self.egraph, AstSize);
    //     let (best_cost, _) = extractor.find_best(self.root_id);
    //     let reward = std::cmp::max(self.last_cost - best_cost, 0); // TODO allow callback cost func

    //     reward as f32
    // }

    pub fn get_action_space(&self) -> usize {
        self.num_rules
    }

    pub fn checkpoint(&self) -> (u32, u32, EGraph<L, N>, Id, usize, usize) {
        (self.cnt, self.sat_counter, self.egraph.clone(), self.root_id.clone(), self.last_cost, self.base_cost)
    }

    pub fn restore(&mut self, checkpoint_data: (u32, u32, EGraph<L, N>, Id, usize, usize) ) {
        (self.cnt, self.sat_counter, self.egraph, self.root_id, self.last_cost, self.base_cost) = checkpoint_data
    }
}
