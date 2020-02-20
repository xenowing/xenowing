#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn todo_something_something() {
        let mut m = Buster::new();

        m.reset();
        m.prop();

        // TODO
    }
}