mod framebuffer;

pub fn init() {
    let mut fb = framebuffer::FrameBuffer::new();
    for i in 0..2 {
        for j in 0..2 {
            fb.write_pixel(framebuffer::Pixel {
                r: 255, g: 255, b: 0, alpha: 200, row: i, column: j
            })
        }
    }
}
