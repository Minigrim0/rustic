struct Pipe {
    buff: Vec<f32>,
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            buff: Vec::new(),
        }
    }

    pub fn push(&mut self, item: f32) {
        self.buff.push(item);
    }

    pub fn pop(&mut self) -> Option<f32> {
        self.buff.pop()
    }

    pub fn take(&mut self, amount: usize) -> Vec<f32> {
        self.buff.drain(0..amount).collect()
    }
}

struct Filter<'a> {
    source: Option<&'a mut Pipe>,
    sink: Option<&'a mut Pipe>,
}

impl Filter<'_> {
    pub fn new() -> Self {
        Self {
            source: None,
            sink: None,
        }
    }

    pub fn set_sink<'a>(&mut self, sink: &'a mut Pipe) {
        self.sink = Some(sink);
    }

    pub fn set_source<'a>(&mut self, source: &'a mut Pipe) {
            self.source = Some(source);
    }

    /// Transfers the
    pub fn transform(&mut self, amount: Option<i32>) {
        let sink = match self.sink {
            Some(s) => s,
            None => {
                println!("No sink, unable to push");
                return
            }
        };
        let source: &mut Pipe = match self.source.as_mut() {
            Some(s) => s,
            None => {
                println!("No source, unable to pull");
                return
            }
        };

        if let Some(amount) = amount {  // Take the given amount
            for item in source.take(amount as usize) {
                sink.push(item * 2.0);
            }
        } else {  // Take everything
            for item in source.buff.iter() {
                sink.push(*item);
            }
        }
    }
}

fn main() {
    let mut source = Pipe::new();
    source.push(1.0);
    source.push(2.0);
    source.push(3.0);

    let mut sink = Pipe::new();

    let mut filter = Filter::new();
    filter.set_source(&source);
    filter.set_sink(&sink);

    for _ in 0..3 {
        filter.transform(None);
    }

    println!("{:?}", sink.pop());
}
