//! Unit tests for the Vantis Kernel module

use vantisweb::core::kernel::VantisKernel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_initialization() {
        let kernel = VantisKernel::new();
        assert!(kernel.is_initialized());
    }

    #[test]
    fn test_kernel_start() {
        let mut kernel = VantisKernel::new();
        assert!(kernel.start().is_ok());
        assert!(kernel.is_running());
    }

    #[test]
    fn test_kernel_stop() {
        let mut kernel = VantisKernel::new();
        kernel.start().unwrap();
        assert!(kernel.stop().is_ok());
        assert!(!kernel.is_running());
    }

    #[test]
    fn test_kernel_restart() {
        let mut kernel = VantisKernel::new();
        kernel.start().unwrap();
        assert!(kernel.restart().is_ok());
        assert!(kernel.is_running());
    }

    #[test]
    fn test_kernel_memory_usage() {
        let kernel = VantisKernel::new();
        let usage = kernel.get_memory_usage();
        assert!(usage > 0);
    }

    #[test]
    fn test_kernel_cpu_usage() {
        let kernel = VantisKernel::new();
        let usage = kernel.get_cpu_usage();
        assert!(usage >= 0.0 && usage <= 100.0);
    }
}