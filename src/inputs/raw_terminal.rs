use std::io;
use std::fs;
use std::os::unix::io::AsRawFd;


pub fn setup_raw_terminal() -> io::Result<()> {
    unsafe {
        let tty;
        let fd = if libc::isatty(libc::STDIN_FILENO) == 1 {
            libc::STDIN_FILENO
        } else {
            tty = fs::File::open("/dev/tty")?;

            tty.as_raw_fd()
        };

        let mut ptr = core::mem::MaybeUninit::uninit();

        if libc::tcgetattr(fd, ptr.as_mut_ptr()) == 0 {
            let mut termios = ptr.assume_init();
            let c_oflag = termios.c_oflag;

            libc::cfmakeraw(&mut termios);
            termios.c_oflag = c_oflag;

            if libc::tcsetattr(fd, libc::TCSADRAIN, &termios) == 0 {
                return Ok(());
            }
        }
    }

    Err(io::Error::last_os_error())
}
