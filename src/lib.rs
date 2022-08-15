mod drum_synth;

#[cfg(test)]
mod tests {
    use crate::generate;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        generate();
        assert_eq!(result, 4);
    }
}

pub fn generate() {
    drum_synth::generate();
}