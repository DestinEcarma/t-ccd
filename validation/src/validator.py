"""Main brute force collision validator class."""


import time
from typing import List

from .analyzer import AccuracyAnalyzer
from .config import (
    max_frames,
    output_dir, 
    time_step,
    progress_report_interval
)
from .models import Collision
from .data_loader import DataLoader
from .detector import CollisionDetector
from .results import ResultsManager


class BruteForceValidator:
    """Main validator class orchestrating the entire validation process."""
    
    def __init__(self, 
                 particles_tccd_file: str = "data/particles_tccd_500.csv",
                 particles_swept_aabb_file: str = "data/particles_swept_aabb_500.csv", 
                 tccd_events_file: str = "data/events_tccd_500.csv",
                 swept_events_file: str = "data/events_swept_aabb_500.csv",
                 output_dir: str = output_dir):
        
        self.data_loader = DataLoader(particles_tccd_file, particles_swept_aabb_file, tccd_events_file, swept_events_file)
        self.results_manager = ResultsManager(output_dir)
        
        print(f"🚀 Brute Force Collision Validator Initialized")
        print(f"📁 Output Directory: {output_dir}")
    
    def run_validation(self, max_frames: int = max_frames, 
                      time_step: float = time_step,
                      use_tccd_particles: bool = False):
        """Run complete brute force validation process."""
        print(f"\n STARTING BRUTE FORCE COLLISION VALIDATION")
        print(f"=" * 60)
        print(f"Max Frames: {max_frames:,}")
        print(f"Time Step: {time_step}")
        print(f"Particle Source: {'T-CCD' if use_tccd_particles else 'Swept AABB'}")
        
        # Load particle states
        particle_states = self.data_loader.load_particle_states(use_tccd_particles, max_frames)
        
        if not particle_states:
            print(" No particle states loaded - cannot proceed")
            return
        
        # Process frames
        all_frames = sorted(particle_states.keys())
        frames_to_process = all_frames[:max_frames] if max_frames else all_frames
        
        print(f" Available frames: {min(all_frames)} to {max(all_frames)} ({len(all_frames):,} total)")
        print(f" Processing {len(frames_to_process):,} frames...")
        
        # Perform brute force collision detection
        ground_truth_collisions = self._detect_all_collisions(
            particle_states, frames_to_process, time_step
        )
        
        # Save results and compare algorithms
        self.results_manager.save_brute_force_results(ground_truth_collisions)
        self._compare_algorithms(ground_truth_collisions, max_frames, use_tccd_particles)
        
        print(f"\n BRUTE FORCE VALIDATION COMPLETED!")
        print(f" Check '{self.results_manager.output_dir}' for detailed results")

    def _detect_all_collisions(self, particle_states, frames_to_process, time_step) -> List[Collision]:
        """Detect all collisions using brute force approach."""
        all_collisions = []
        start_time = time.time()
        
        for frame_idx, frame in enumerate(frames_to_process):
            particles = particle_states[frame]
            frame_collisions = self._detect_frame_collisions(particles, time_step)
            all_collisions.extend(frame_collisions)
            
            # Progress reporting
            if (frame_idx + 1) % progress_report_interval == 0 or frame_idx < 10:
                self._report_progress(frame_idx, frame, len(frame_collisions), 
                                    len(frames_to_process), start_time)
        
        total_time = time.time() - start_time
        print(f"  Brute force detection completed in {total_time:.2f}s")
        print(f" Total collisions detected: {len(all_collisions):,}")
        
        return all_collisions
    
    def _detect_frame_collisions(self, particles, time_step) -> List[Collision]:
        """Detect collisions for a single frame using O(n²) comparison."""
        collisions = []
        n = len(particles)
        
        for i in range(n):
            for j in range(i + 1, n):
                collision = CollisionDetector.detect_collision(
                    particles[i], particles[j], time_step
                )
                if collision:
                    collisions.append(collision)
        
        return collisions
    
    def _report_progress(self, frame_idx: int, frame: int, collisions_count: int,
                        total_frames: int, start_time: float):
        """Report validation progress."""
        elapsed = time.time() - start_time
        progress = (frame_idx + 1) / total_frames * 100
        eta = elapsed / (frame_idx + 1) * total_frames - elapsed
        
        print(f"   Frame {frame}: {collisions_count:,} collisions "
              f"({progress:.1f}% complete, ETA: {eta:.1f}s)")
    
    def _compare_algorithms(self, ground_truth_collisions: List[Collision], 
                           max_frames: int, use_tccd_particles: bool = False):
        """Compare algorithm accuracy against ground truth."""
        print(f"\n ALGORITHM ACCURACY COMPARISON")
        print(f"=" * 50)
        
        # Load algorithm events
        tccd_events, swept_events = self.data_loader.load_algorithm_events(max_frames)
        
        if tccd_events.empty and swept_events.empty:
            print(" No algorithm events loaded - cannot compare")
            return
        
        # Create ground truth signature set
        ground_truth_set = AccuracyAnalyzer.create_collision_signature_set(ground_truth_collisions)
        print(f" Ground Truth: {len(ground_truth_set):,} unique collisions")
        
        # Compare algorithms
        tccd_results = None
        swept_results = None
        
        if use_tccd_particles:
            # Using T-CCD particles -> only validate T-CCD algorithm
            print(f"Validating T-CCD algorithm (using T-CCD particle trajectories)")
            if not tccd_events.empty:
                tccd_results = AccuracyAnalyzer.compare_algorithm_events(
                    tccd_events, ground_truth_set, "T-CCD"
                )
            else:
                print("No T-CCD events found for validation")
        else:
            # Using Swept AABB particles -> only validate Swept AABB algorithm  
            print(f"Validating Swept AABB algorithm (using Swept AABB particle trajectories)")
            if not swept_events.empty:
                swept_results = AccuracyAnalyzer.compare_algorithm_events(
                    swept_events, ground_truth_set, "Swept AABB"
                )
            else:
                print(" No Swept AABB events found for validation")
        
        # Save comparison results
        self.results_manager.save_accuracy_comparison(
            tccd_results, swept_results, len(ground_truth_set)
        )