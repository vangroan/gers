//! Poisson disc sampling.
use crate::collections::F64Array;
use arrayvec::ArrayVec;
use rand::prelude::*;
use rand_xorshift::XorShiftRng;
use rust_wren::prelude::*;
use std::f64::consts::PI;

const DEFAULT_SEED: u32 = 0;

/// See: https://a5huynh.github.io/posts/2019/poisson-disk-sampling/
#[wren_class]
#[derive(Debug)]
pub struct PoissonDisc {
    options: PoissonOptions,
    /// Width and height of a single cell in the grid.
    cell_size: f64,
    /// Grid dimensions as width and height.
    grid_dim: [f64; 2],
    /// Grid data. Cells can be empty, [`None`], or sampled. Only one
    /// sample can occupy a cell.
    grid: Vec<Option<[usize; 2]>>,
    /// Active list of samples that possibly have space around them
    /// to place new samples.
    active: Vec<[usize; 2]>,
    /// All the generated valid samples.
    samples: Vec<[usize; 2]>,
}

#[derive(Debug)]
pub struct PoissonOptions {
    /// Random number generator seed. When [`None`] defaults to internal RNG behaviour.
    pub seed: Option<u32>,
    /// Minimum distance radius between samples.
    pub min_distance: u32,
    /// Number of samples to attempt before rejection.
    pub num_samples: usize,
    /// Width and height of area to generate samples in.
    pub dimensions: [u32; 2],
    pub padding: [usize; 2],
}

#[wren_methods]
impl PoissonDisc {
    #[construct]
    fn new_() -> Self {
        Default::default()
    }

    #[method(name = new)]
    fn new_seed(seed: u32) -> Self {
        Self::new(PoissonOptions {
            seed: Some(seed),
            ..PoissonOptions::default()
        })
    }

    #[method(name = new)]
    fn new_3(seed: u32, width: u32, height: u32) -> Self {
        Self::new(PoissonOptions {
            seed: Some(seed),
            dimensions: [width, height],
            ..PoissonOptions::default()
        })
    }

    #[method(name = new)]
    fn new_4(seed: u32, min_distance: u32, width: u32, height: u32) -> Self {
        Self::new(PoissonOptions {
            seed: Some(seed),
            min_distance,
            dimensions: [width, height],
            ..PoissonOptions::default()
        })
    }

    #[method(name = generate)]
    fn generate_(&mut self) -> F64Array {
        self.generate();
        let vector = self
            .samples
            .drain(..)
            .flat_map(ArrayVec::from)
            .map(|component| component as f64)
            .collect::<Vec<f64>>();

        F64Array::from(vector)
    }
}

impl PoissonDisc {
    pub fn new(options: PoissonOptions) -> Self {
        let [width, height] = options.dimensions;

        // Each cell can contain only one sample. The longest
        // possible distance is the diagonal from one corner to
        // the oposite corner. The diagonal line, r, can be
        // calculated using the Pythagoras theorem.
        //
        //      x² + y² = r²
        //
        // We need the dimensions of the cells. Cells are square,
        // with and height are equal, so we can substitute y with x.
        //
        //      x² + x² = r²
        //          2x² = r²
        //
        // Solve for x.
        //
        //            x = r / √2
        let cell_size = options.min_distance as f64 / 2.0_f64.sqrt();

        // TODO: Explain why + 1.0
        let grid_dim = [
            (width as f64 / cell_size).ceil() + 1.0,
            (height as f64 / cell_size).ceil() + 1.0,
        ];

        // Random number generator.
        // let mut rng = rand::thread_rng();
        let seed = options.seed.unwrap_or(DEFAULT_SEED);
        let mut rng = Self::create_rng(seed);

        let mut poisson = PoissonDisc {
            options,
            grid_dim,
            cell_size,
            grid: vec![None; (grid_dim[0] * grid_dim[1]) as usize],
            active: vec![],
            samples: vec![],
        };

        // Initial sample to act as a starting point for the rest of the samples.
        let point = [
            (rng.gen::<f64>() * width as f64) as usize,
            (rng.gen::<f64>() * height as f64) as usize,
        ];
        poisson.insert_point(point);
        poisson.active.push(point);
        poisson.samples.push(point);

        poisson
    }

    /// Given a point, return the index in the flat one dimensional
    /// vector used as storage for the grid.
    fn index(&self, point: [usize; 2]) -> usize {
        // Calculate cell coordinate in grid space.
        let x = (point[0] as f64 / self.cell_size).floor();
        let y = (point[1] as f64 / self.cell_size).floor();
        log::trace!(
            "index(); point={:?}; coords={:?}; index={:?}",
            point,
            [x, y],
            (x + y * self.grid_dim[0]) as usize
        );

        // Calculate index in the flat array.
        (x + y * self.grid_dim[0]) as usize
    }

    fn insert_point(&mut self, point: [usize; 2]) {
        let index = self.index(point);
        self.grid[index] = Some(point);
    }

    /// Indicates whether a point occupies a grid cell.
    #[allow(dead_code)]
    fn is_occupied(&self, point: [usize; 2]) -> bool {
        let index = self.index(point);
        self.grid[index].is_some()
    }

    /// Generate a random sample in range of the given point.
    fn sample<T: Rng>(&mut self, rng: &mut T, point: [usize; 2]) -> [usize; 2] {
        // Random angle between 0 and 260 degrees.
        let angle = 2.0 * PI * rng.gen::<f64>();

        // Random radius between r and 2r.
        let radius = self.options.min_distance as f64 * (rng.gen::<f64>() + 1.0);

        // Convert polar coordinates to cartesian coordinates.
        let x = point[0] as f64 + (radius * angle.cos());
        let y = point[1] as f64 + (radius * angle.sin());

        // Clamp the coordinate to the grid bounds.
        let [width, height] = self.options.dimensions;
        let [padx, pady] = self.options.padding;
        [
            x.max(padx as f64).min(width as f64 - padx as f64) as usize,
            y.max(pady as f64).min(height as f64 - pady as f64) as usize,
        ]
    }

    /// Checks whether the given point is valid in the grid.
    fn is_valid(&self, point: [usize; 2]) -> bool {
        // Scale the source point onto the grid.
        let xidx = (point[0] as f64 / self.cell_size).floor();
        let yidx = (point[1] as f64 / self.cell_size).floor();

        // Determine the neighborhood around the source point.
        // The disc extends to a maximum of two cells away.
        let [grid_width, grid_height] = self.grid_dim;
        let start_x = (xidx - 2.0).max(0.0) as usize;
        let end_x = (xidx + 2.0).min(grid_width - 1.0) as usize;
        let start_y = (yidx - 2.0).max(0.0) as usize;
        let end_y = (yidx + 2.0).min(grid_height - 1.0) as usize;

        // Check all non-empty neighbouring cells and make sure the new point
        // is outside their radius.
        for y in start_y..end_y {
            for x in start_x..end_x {
                let index = x + y * grid_width as usize;
                if let Some(cell) = self.grid[index] {
                    if distance(cell, point) <= self.options.min_distance as f64 {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn create_rng(seed: u32) -> XorShiftRng {
        let mut real = [0; 16];
        real[0] = 1;
        for i in 1..4 {
            real[i * 4] = seed as u8;
            real[(i * 4) + 1] = (seed >> 8) as u8;
            real[(i * 4) + 2] = (seed >> 16) as u8;
            real[(i * 4) + 3] = (seed >> 24) as u8;
        }
        XorShiftRng::from_seed(real)
    }

    pub fn generate(&mut self) {
        log::debug!("Generating poisson disc blue noise. {:?}", self.options);
        let seed = self.options.seed.unwrap_or(DEFAULT_SEED);
        let mut rng = Self::create_rng(seed);
        // let mut rng = rand::thread_rng();

        while !self.active.is_empty() {
            let index = rng.gen::<usize>() % self.active.len();
            let source = self.active[index];

            let mut found = false;
            for _ in 0..self.options.num_samples {
                let point = self.sample(&mut rng, source);

                if self.is_valid(point) {
                    self.insert_point(point);
                    self.active.push(point);
                    self.samples.push(point);
                    found = true;
                }
            }

            if !found {
                self.active.remove(index);
            }
        }
    }

    /// Consume the sampler and return the generated sample points.
    #[allow(dead_code)]
    pub fn take_samples(self) -> Vec<[usize; 2]> {
        let PoissonDisc { samples, .. } = self;
        samples
    }
}

impl Default for PoissonDisc {
    fn default() -> Self {
        PoissonDisc::new(PoissonOptions::default())
    }
}

impl Default for PoissonOptions {
    fn default() -> Self {
        PoissonOptions {
            seed: None,
            min_distance: 1,
            num_samples: 30,
            dimensions: [100, 100],
            padding: [10, 10],
        }
    }
}

fn distance(a: [usize; 2], b: [usize; 2]) -> f64 {
    let delta_x = b[0] as f64 - a[0] as f64;
    let delta_y = b[1] as f64 - a[1] as f64;

    // Euclidean distance.
    (delta_x.powf(2.0) + delta_y.powf(2.0)).sqrt()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_insert_point() {
        let mut poi = PoissonDisc::default();

        poi.insert_point([25, 25]);
        assert!(poi.is_occupied([25, 25]));
        assert!(!poi.is_occupied([26, 25]));
    }
}
