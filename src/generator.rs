use crate::{
    config::GenerationConfig,
    kernel::Kernel,
    map::{BlockType, Map},
    position::Position,
    random::Random,
    walker::CuteWalker,
};

use ndarray::{s, Array2};

pub struct Generator {
    pub walker: CuteWalker,
    pub map: Map,
    pub rnd: Random,
}

impl Generator {
    /// derive a initial generator state based on a GenerationConfig
    pub fn new(config: &GenerationConfig, seed: u64) -> Generator {
        let spawn = Position::new(50, 250);
        let map = Map::new(300, 300, BlockType::Hookable, spawn.clone());
        let init_inner_kernel = Kernel::new(config.inner_size_bounds.1, 0.0);
        let init_outer_kernel = Kernel::new(config.outer_size_bounds.1, 0.1);
        let walker = CuteWalker::new(spawn, init_inner_kernel, init_outer_kernel, config);
        let rnd = Random::new(seed, config.step_weights.clone());

        Generator { walker, map, rnd }
    }

    pub fn step(&mut self, config: &GenerationConfig) -> Result<(), &'static str> {
        // check if walker has reached goal position
        if self.walker.is_goal_reached() == Some(true) {
            self.walker.next_waypoint();
        }

        if !self.walker.finished {
            // randomly mutate kernel
            self.walker.mutate_kernel(config, &mut self.rnd);

            // perform one step
            self.walker
                .probabilistic_step(&mut self.map, &mut self.rnd)?;
        }

        Ok(())
    }

    /// Post processing step to fix all existing edge-bugs, as certain inner/outer kernel
    /// configurations do not ensure a min. 1-block freeze padding consistently.
    fn fix_edge_bugs(&mut self) -> Array2<bool> {
        let mut edge_bug = Array2::from_elem((self.map.width, self.map.height), false);
        let width = self.map.width;
        let height = self.map.height;

        for x in 0..width {
            for y in 0..height {
                let value = &self.map.grid[[x, y]];
                if *value == BlockType::Empty {
                    for dx in 0..=2 {
                        for dy in 0..=2 {
                            if dx == 1 && dy == 1 {
                                continue;
                            }

                            let neighbor_x = x + dx - 1; // TODO: deal with overflow?
                            let neighbor_y = y + dy - 1;
                            if neighbor_x < width && neighbor_y < height {
                                let neighbor_value = &self.map.grid[[neighbor_x, neighbor_y]];
                                if *neighbor_value == BlockType::Hookable {
                                    edge_bug[[x, y]] = true;
                                    // break;
                                }
                            }
                        }
                    }

                    if edge_bug[[x, y]] {
                        self.map.grid[[x, y]] = BlockType::Freeze;
                    }
                }
            }
        }

        edge_bug
    }

    fn generate_room(&mut self, pos: &Position, margin: usize, zone_type: &BlockType) {
        // TODO: ensure valid position?

        // carve room
        self.map.set_area(
            &Position::new(pos.x - margin, pos.y - margin),
            &Position::new(pos.x + margin, pos.y + margin),
            &BlockType::Empty,
            true,
        );

        // set platform
        self.map.set_area(
            &Position::new(pos.x - (margin - 2), pos.y),
            &Position::new(pos.x + (margin - 2), pos.y),
            &BlockType::Hookable,
            true,
        );

        // set spawns
        if *zone_type == BlockType::Start {
            self.map.set_area(
                &Position::new(pos.x - (margin - 2), pos.y - 1),
                &Position::new(pos.x + (margin - 2), pos.y - 1),
                &BlockType::Spawn,
                true,
            );
        }

        // set start/finish line
        self.map.set_area_border(
            &Position::new(pos.x - margin - 1, pos.y - margin - 1),
            &Position::new(pos.x + margin + 1, pos.y + margin + 1),
            zone_type,
            false,
        );
    }

    pub fn post_processing(&mut self) {
        self.fix_edge_bugs();
        self.generate_room(&self.map.spawn.clone(), 4, &BlockType::Start);
        self.generate_room(&self.walker.pos.clone(), 4, &BlockType::Finish);
    }

    /// Generates an entire map with a single function call. This function is used by the CLI.
    /// It is important to keep this function up to date with the editor generation, so that
    /// fixed seed map generations result in the same map.
    pub fn generate_map(
        max_steps: usize,
        seed: u64,
        config: &GenerationConfig,
    ) -> Result<Map, &'static str> {
        let mut gen = Generator::new(&config, seed);

        for _ in 0..max_steps {
            if gen.walker.finished {
                break;
            }
            gen.step(&config)?;
        }

        gen.post_processing();

        Ok(gen.map)
    }
}
