pub enum Distribution {
    Discrete(Discrete),
    Uniform(Uniform),
    Exponential(Exponential),
    Gaussian(Gaussian),
}

pub struct Discrete {
    outcomes: Vec<f64>,
    probabilities: Vec<f64>,
}

pub struct Uniform {
    lower: f64,
    upper: f64,
}

pub struct Exponential {
    decay: f64,
    lower: f64,
    upper: f64,
}

pub struct Gaussian {
    mean: f64,
    std: f64,
    lower: f64,
    upper: f64,
}
