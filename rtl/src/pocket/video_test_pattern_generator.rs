use kaze::*;

pub struct VideoTestPatternGenerator<'a> {
    pub m: &'a Module<'a>,

    pub system_write_reset_pulse: &'a Input<'a>,
    pub system_write_line_pulse: &'a Input<'a>,

    pub video_line_buffer_write_enable: &'a Output<'a>,
    pub video_line_buffer_write_addr: &'a Output<'a>,
    pub video_line_buffer_write_data: &'a Output<'a>,
}

impl<'a> VideoTestPatternGenerator<'a> {
    pub fn new(
        instance_name: impl Into<String>,
        p: &'a impl ModuleParent<'a>,
    ) -> VideoTestPatternGenerator<'a> {
        let m = p.module(instance_name, "VideoTestPatternGenerator");

        let video_width = 320u32;
        let video_height = 240u32;

        let test_pattern_x = m.reg("test_pattern_x", 9);
        let test_pattern_y = m.reg("test_pattern_y", 8);

        let test_pattern_x_start = test_pattern_x.eq(m.lit(0u32, 9));
        let test_pattern_x_end = test_pattern_x.eq(m.lit(video_width - 1, 9));
        let test_pattern_y_start = test_pattern_y.eq(m.lit(0u32, 8));
        let test_pattern_y_end = test_pattern_y.eq(m.lit(video_height - 1, 8));

        let system_write_reset_pulse = m.input("system_write_reset_pulse", 1);
        let system_write_line_pulse = m.input("system_write_line_pulse", 1);

        let video_line_buffer_write_enable_reg = m.reg("video_line_buffer_write_enable_reg", 1);
        video_line_buffer_write_enable_reg.default_value(false);

        let video_line_buffer_write_addr_reg = m.reg("video_line_buffer_write_addr_reg", 10);

        test_pattern_x.drive_next(
            if_(system_write_line_pulse, m.lit(0u32, 9))
                .else_if(
                    video_line_buffer_write_enable_reg,
                    test_pattern_x + m.lit(1u32, 9),
                )
                .else_(test_pattern_x),
        );

        test_pattern_y.drive_next(
            if_(system_write_reset_pulse, m.lit(0u32, 8))
                .else_if(
                    video_line_buffer_write_enable_reg & test_pattern_x_end,
                    test_pattern_y + m.lit(1u32, 8),
                )
                .else_(test_pattern_y),
        );

        video_line_buffer_write_enable_reg.drive_next(
            if_(system_write_reset_pulse, m.lit(false, 1))
                .else_if(system_write_line_pulse, m.lit(true, 1))
                .else_if(
                    video_line_buffer_write_enable_reg & test_pattern_x_end,
                    m.lit(false, 1),
                )
                .else_(video_line_buffer_write_enable_reg),
        );

        video_line_buffer_write_addr_reg.drive_next(
            if_(system_write_reset_pulse, m.lit(0u32, 10))
                .else_if(
                    video_line_buffer_write_enable_reg,
                    if_(
                        video_line_buffer_write_addr_reg.eq(m.lit(video_width * 2 - 1, 10)),
                        m.lit(0u32, 10),
                    )
                    .else_(video_line_buffer_write_addr_reg + m.lit(1u32, 10)),
                )
                .else_(video_line_buffer_write_addr_reg),
        );

        let video_line_buffer_write_data = if_(
            test_pattern_x_start | test_pattern_x_end | test_pattern_y_start | test_pattern_y_end,
            m.lit(0xffu32, 8).repeat(3),
        )
        .else_(
            test_pattern_x
                .bits(7, 0)
                .concat(test_pattern_y)
                .concat(m.lit(64u32, 8)),
        );

        VideoTestPatternGenerator {
            m,

            system_write_reset_pulse,
            system_write_line_pulse,

            video_line_buffer_write_enable: m.output(
                "video_line_buffer_write_enable",
                video_line_buffer_write_enable_reg,
            ),
            video_line_buffer_write_addr: m.output(
                "video_line_buffer_write_addr",
                video_line_buffer_write_addr_reg,
            ),
            video_line_buffer_write_data: m
                .output("video_line_buffer_write_data", video_line_buffer_write_data),
        }
    }
}