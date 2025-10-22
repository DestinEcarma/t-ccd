"""Command-line interface for T-CCD validation framework."""

import argparse
import sys

from .config import max_frames, time_step 
from .validation.validator import BruteForceValidator


def main():
    """Main entry point for T-CCD validation CLI."""
    parser = argparse.ArgumentParser(
        description="T-CCD: Brute Force Collision Detection Validator"
    )
    
    parser.add_argument(
        "--max-frames", 
        type=int, 
        default=max_frames,
        help=f"Maximum number of frames to process (default: {max_frames})"
    )
    
    parser.add_argument(
        "--time-step",
        type=float,
        default=time_step,
        help=f"Time step for simulation in seconds (default: {time_step})"
    )
    
    parser.add_argument(
        "--particles-tccd-file",
        type=str,
        default="data/particles_tccd_500.csv",
        help="Path to T-CCD particles CSV file"
    )
    
    parser.add_argument(
        "--particles-swept-aabb-file",
        type=str,
        default="data/particles_swept_aabb_500.csv",
        help="Path to Swept AABB particles CSV file"
    )
    
    parser.add_argument(
        "--use-tccd-particles",
        action="store_true",
        help="Use T-CCD particles for ground truth (default: use Swept AABB particles)"
    )
    
    parser.add_argument(
        "--tccd-events-file",
        type=str,
        default="data/events_tccd_500.csv",
        help="Path to T-CCD events CSV file"
    )
    
    parser.add_argument(
        "--swept-events-file",
        type=str,
        default="data/events_swept_aabb_500.csv",
        help="Path to Swept AABB events CSV file"
    )
    
    parser.add_argument(
        "--output-dir",
        type=str,
        default="brute_force_validation",
        help="Output directory for validation results"
    )
    
    args = parser.parse_args()
    
    print(" T-CCD: Brute Force Collision Detection Validator")
    print("=" * 60)
    print("This tool will:")
    print("1. Load your exact particle states from CSV files")
    print("2. Perform exhaustive collision detection (ground truth)")
    print("3. Compare T-CCD and Swept AABB accuracy against ground truth")
    print("4. Generate detailed accuracy metrics and save to CSV")
    print("=" * 60)
    
    print(f" Configuration:")
    print(f"   Max Frames: {args.max_frames:,}")
    print(f"   Time Step: {args.time_step}")
    print(f"   Output Dir: {args.output_dir}")
    print(f"   Particle Source: {'T-CCD' if args.use_tccd_particles else 'Swept AABB'}")
    print(f"   T-CCD Particles: {args.particles_tccd_file}")
    print(f"   Swept AABB Particles: {args.particles_swept_aabb_file}")
    print()
    
    try:
        validator = BruteForceValidator(
            particles_tccd_file=args.particles_tccd_file,
            particles_swept_aabb_file=args.particles_swept_aabb_file,
            tccd_events_file=args.tccd_events_file,
            swept_events_file=args.swept_events_file,
            output_dir=args.output_dir
        )
        
        validator.run_validation(
            max_frames=args.max_frames,
            time_step=args.time_step,
            use_tccd_particles=args.use_tccd_particles
        )
        
    except Exception as e:
        print(f"\n Validation failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()