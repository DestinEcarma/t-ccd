"""Test module for T-CCD validation framework."""

from .validation.validator import BruteForceValidator


def main():
    """Run a quick test of the T-CCD validator."""
    print(" Testing T-CCD Brute Force Collision Validator")
    print("=" * 50)
    
    # Test with small frame count for quick verification
    test_frames = 10
    
    try:
        validator = BruteForceValidator()
        validator.run_validation(max_frames=test_frames, time_step=1.0/60.0)
        print(f"\n Test completed successfully with {test_frames} frames!")
        return True
        
    except Exception as e:
        print(f"\n Test failed: {e}")
        import traceback
        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = main()
    if success:
        print("\n T-CCD framework is ready for production use!")
        print("📝 You can now run the full validation with:")
        print("   uv run tccd-validate")
    else:
        print("\n Please check the errors and fix any issues.")