extern crate ranking;

use ranking::compute_ratings::{simulate_contest, RatingSystem};
use ranking::contest_config::{get_contest, get_contest_config, get_contest_ids, ContestSource};
use ranking::metrics::compute_metrics_custom;
use std::collections::HashMap;
use std::time::Instant;

#[allow(unused_imports)]
use ranking::{CodeforcesSystem, EloRSystem, TopCoderSystem, TrueSkillSPBSystem};

fn main() {
    // Prepare the contest system parameters
    let mut systems: Vec<Box<dyn RatingSystem>> = vec![];
    for si in (100..=500).step_by(40) {
        for wi in -8..=4 {
            let sig_perf = si as f64;
            let weight = 10f64.powf((wi as f64) * 0.25);
            let system = CodeforcesSystem { sig_perf, weight };
            systems.push(Box::new(system));
        }
    }
    for pi in (100..=500).step_by(50) {
        for li in (0..=120).step_by(20) {
            let sig_perf = pi as f64;
            let sig_drift = li as f64;
            let system = EloRSystem {
                sig_perf,
                sig_drift,
                split_ties: false,
                variant: ranking::EloRVariant::Gaussian,
            };
            systems.push(Box::new(system));
        }
    }
    for ri in -1..=1 {
        for pi in (100..=500).step_by(50) {
            for li in (0..=120).step_by(20) {
                let sig_perf = pi as f64;
                let sig_drift = li as f64;
                let system = EloRSystem {
                    sig_perf,
                    sig_drift,
                    split_ties: false,
                    variant: ranking::EloRVariant::Logistic(2f64.powi(ri)),
                };
                systems.push(Box::new(system));
            }
        }
    }
    for wi in -15..=15 {
        let weight_multiplier = 10f64.powf((wi as f64) * 0.1);
        let system = TopCoderSystem { weight_multiplier };
        systems.push(Box::new(system));
    }
    for ei in 1..=5 {
        for bi in (140..=360).step_by(40) {
            for si in (0..=20).step_by(4) {
                let eps = (ei as f64) * 0.1;
                let beta = bi as f64;
                let sigma_growth = si as f64;
                let system = TrueSkillSPBSystem {
                    eps,
                    beta,
                    convergence_eps: 2e-4,
                    sigma_growth,
                };
                systems.push(Box::new(system));
            }
        }
    }

    // Run the contest histories and measure
    let config = get_contest_config(ContestSource::Codeforces);
    let contest_ids = get_contest_ids(&config.contest_id_file);
    let max_contests = usize::MAX;
    let mu_noob = 1500.;
    let sig_noob = 300.;
    for system in systems {
        let mut players = HashMap::new();
        let mut avg_perf = compute_metrics_custom(&mut players, &[]);
        let now = Instant::now();

        for &contest_id in contest_ids.iter().take(max_contests) {
            let contest = get_contest(&config.contest_cache_folder, contest_id);

            // Predict performance must be run before simulate contest
            // since we don't want to make predictions after we've seen the contest
            avg_perf += compute_metrics_custom(&mut players, &contest.standings);

            // Now run the actual rating update
            simulate_contest(&mut players, &contest, &*system, mu_noob, sig_noob);
        }
        println!(
            "{:?}: {}, {}s",
            system,
            avg_perf,
            now.elapsed().as_millis() as f64 / 1000.
        );
    }
}