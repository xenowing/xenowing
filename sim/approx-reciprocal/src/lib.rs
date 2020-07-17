#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    fn test(x: u32, expected_quotient: u32, div_shiz: &mut ApproxReciprocal) {
        div_shiz.x = x;
        div_shiz.prop();
        for i in 0..13 {
            div_shiz.posedge_clk();
            // Feed bogus value in between real ones to test that everything's pipelined correctly
            div_shiz.x = 0xfadebabe;
            div_shiz.prop();
        }
        assert_eq!(expected_quotient, div_shiz.quotient);
    }

    #[test]
    fn random_rasterizer_reference_values() {
        let mut div_shiz = ApproxReciprocal::new();

        test(0x0a4e6be0, 0x00000635, &mut div_shiz);
        test(0x40000000, 0x000000ff, &mut div_shiz);
        test(0x0a7bb1e0, 0x0000061a, &mut div_shiz);
        test(0x0add6a90, 0x000005e3, &mut div_shiz);
    }
}
