use crate::env::Info;
use egg::{
    Analysis, AstSize, EGraph, Extractor, Id, Language, RecExpr, Rewrite, Runner, SimpleScheduler,
    StopReason,
};
use std::time::Duration;

#[derive(Clone)]
pub struct Ckpt<L, N>
where
    L: Language + 'static + egg::FromOp + std::marker::Send,
    N: Analysis<L> + Clone + 'static + std::default::Default + std::marker::Send,
    N::Data: Clone,
    <N as Analysis<L>>::Data: Send,
{
    pub cnt: u32,
    pub sat_counter: usize,
    pub egraph: EGraph<L, N>,
    pub root_id: Id,
    pub last_cost: usize,
    // debug term
    // pub egraph_nodes: usize,
    // pub egraph_classes: usize,
    // pub memo_size: usize,
}

pub struct EgraphEnv<L, N>
where
    L: Language + 'static + egg::FromOp + std::marker::Send,
    N: Analysis<L> + Clone + 'static + std::default::Default + std::marker::Send,
    N::Data: Clone,
    <N as Analysis<L>>::Data: Send,
{
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

impl<L, N> EgraphEnv<L, N>
where
    L: Language + 'static + egg::FromOp + std::marker::Send,
    N: Analysis<L> + Clone + 'static + std::default::Default + std::marker::Send,
    N::Data: Clone,
    <N as Analysis<L>>::Data: Send,
{
    pub fn new(
        expr: RecExpr<L>,
        rules: Vec<Rewrite<L, N>>,
        node_limit: usize,
        time_limit: usize,
    ) -> Self {
        let runner: Runner<L, N> = Runner::default().with_expr(&expr);
        let root = runner.roots[0];
        let (base_cost, _) = Extractor::new(&runner.egraph, AstSize).find_best(root);
        EgraphEnv {
            expr: expr,
            egraph: EGraph::default(),
            root_id: root,
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
        self.egraph.add_expr(&self.expr);
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
                // TODO this indicates egraph is exploded?
                done = true;
                println!(
                    "EGG TimeLimit {}s - {}s - {} - {} - {}",
                    time,
                    report.total_time,
                    report.egraph_nodes,
                    report.egraph_classes,
                    report.memo_size,
                );
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
        // compute reward
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

    pub fn checkpoint(&self) -> Ckpt<L, N> {
        Ckpt {
            cnt: self.cnt,
            sat_counter: self.sat_counter,
            egraph: self.egraph.clone(),
            root_id: self.root_id.clone(),
            last_cost: self.last_cost,
            // egraph_nodes: self.egraph.total_number_of_nodes(),
            // egraph_classes: self.egraph.number_of_classes(),
            // memo_size: self.egraph.total_size(),
        }
    }

    pub fn restore(&mut self, checkpoint_data: Ckpt<L, N>) {
        self.cnt = checkpoint_data.cnt;
        self.sat_counter = checkpoint_data.sat_counter;
        self.egraph = checkpoint_data.egraph;
        self.root_id = checkpoint_data.root_id;
        self.last_cost = checkpoint_data.last_cost;
        // debug
        // print!("[Debug] {} - {} - {}\t", checkpoint_data.egraph_nodes, checkpoint_data.egraph_classes, checkpoint_data.memo_size);
        // print!("[Self] {} - {} - {}\t", self.egraph.total_number_of_nodes(), self.egraph.number_of_classes(), self.egraph.total_size());
        // println!("[ROOT] {}", self.root_id);
    }
}
