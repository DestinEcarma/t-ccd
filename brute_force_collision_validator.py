
import numpy as np
import pandas as pd
import os
import time
from datetime import datetime
from typing import List, Dict, Tuple, Set
from dataclasses import dataclass
from collections import defaultdict
import math

@dataclass
class Particle:
    """Particle state at a specific frame"""
    particle_id: int
    frame: int
    x: float
    y: float
    vx: float
    vy: float
    radius: float
    mass: float = 1.0

@dataclass 
class Collision:
    """Collision event detected by brute force"""
    frame: int
    time_s: float
    toi: float  # Time of impact within frame
    particle1: int
    particle2: int
    x: float  # Collision position
    y: float
    nx: float  # Normal vector components
    ny: float
    distance: float  # Distance between centers at collision
    relative_velocity: float

class BruteForceValidator:
    """Brute force collision detection validator"""
    
    def __init__(self, 
                 particles_tccd_file: str = "particles_tccd_500.csv",
                 particles_swept_file: str = "particles_swept_aabb_500.csv", 
                 events_tccd_file: str = "events_tccd_500.csv",
                 events_swept_file: str = "events_swept_aabb_500.csv",
                 output_dir: str = "brute_force_validation"):
        
        self.particles_tccd_file = particles_tccd_file
        self.particles_swept_file = particles_swept_file
        self.events_tccd_file = events_tccd_file
        self.events_swept_file = events_swept_file
        self.output_dir = output_dir
        
        # Ensure output directory exists
        os.makedirs(output_dir, exist_ok=True)
        
        print(f" Brute Force Collision Validator Initialized")
        print(f" Output Directory: {output_dir}")
    
    def load_particle_states(self, max_frames: int = None) -> Dict[int, List[Particle]]:
        """
        Load particle states from CSV files
        Returns: Dict[frame_number] -> List[Particle]
        """
        print(" Loading particle states...")
        
        # Try to load T-CCD particles first (as primary source)
        particle_states = {}
        
        try:
            print(f"   Loading from: {self.particles_tccd_file}")
            
            # Load in chunks to handle large files
            chunk_size = 100000
            frame_count = 0
            
            for chunk in pd.read_csv(self.particles_tccd_file, chunksize=chunk_size):
                for _, row in chunk.iterrows():
                    frame = int(row['frame'])
                    
                    # Limit frames if specified
                    if max_frames is not None and frame > max_frames:
                        break
                        
                    if frame not in particle_states:
                        particle_states[frame] = []
                    
                    particle = Particle(
                        particle_id=int(row['particle_id']),
                        frame=frame,
                        x=float(row['x']),
                        y=float(row['y']),
                        vx=float(row['vx']),
                        vy=float(row['vy']),
                        radius=float(row['radius']),
                        mass=float(row.get('mass', 1.0))
                    )
                    
                    particle_states[frame].append(particle)
                
                frame_count = max(particle_states.keys()) if particle_states else 0
                if max_frames is not None and frame_count >= max_frames:
                    break
            
            print(f"    Loaded {len(particle_states)} frames")
            if particle_states:
                sample_frame = min(particle_states.keys())
                particle_count = len(particle_states[sample_frame])
                print(f"    ~{particle_count} particles per frame")
            
            return particle_states
            
        except Exception as e:
            print(f"    Error loading particles: {e}")
            return {}
    
    def detect_collisions_brute_force(self, particles: List[Particle], 
                                    frame: int, time_step: float = 0.1) -> List[Collision]:
        """
        Perform exhaustive O(n²) collision detection for a single frame
        
        Args:
            particles: List of particles in current frame
            frame: Frame number
            time_step: Time step size for this frame
            
        Returns:
            List of detected collisions
        """
        collisions = []
        n = len(particles)
        
        # Check every particle pair
        for i in range(n):
            for j in range(i + 1, n):
                p1, p2 = particles[i], particles[j]
                
                # Calculate collision detection
                collision = self._check_particle_collision(p1, p2, frame, time_step)
                if collision:
                    collisions.append(collision)
        
        return collisions
    
    def _check_particle_collision(self, p1: Particle, p2: Particle, 
                                frame: int, time_step: float) -> Collision:
        """
        Check if two particles collide using continuous collision detection
        
        Uses quadratic formula to solve for exact collision time:
        |p1(t) - p2(t)| = r1 + r2
        
        Returns collision object if collision occurs, None otherwise
        """
        
        # Relative position and velocity
        dx = p1.x - p2.x
        dy = p1.y - p2.y
        dvx = p1.vx - p2.vx
        dvy = p1.vy - p2.vy
        
        # Combined radius
        r_sum = p1.radius + p2.radius
        
        # Quadratic equation coefficients: a*t² + b*t + c = 0
        a = dvx * dvx + dvy * dvy
        b = 2.0 * (dx * dvx + dy * dvy)
        c = dx * dx + dy * dy - r_sum * r_sum
        
        # Check if particles are moving apart (b > 0)
        if b > 0:
            return None
            
        # Solve quadratic equation
        discriminant = b * b - 4.0 * a * c
        
        if discriminant < 0:
            # No collision
            return None
        
        if abs(a) < 1e-10:
            # Particles have same velocity - check if already overlapping
            if c <= 0:
                # Already overlapping - report collision at t=0
                t = 0.0
            else:
                return None
        else:
            # Two solutions: t1 = (-b - sqrt(d))/(2a), t2 = (-b + sqrt(d))/(2a)
            sqrt_d = math.sqrt(discriminant)
            t1 = (-b - sqrt_d) / (2.0 * a)
            t2 = (-b + sqrt_d) / (2.0 * a)
            
            # We want the first collision time (smallest positive t)
            if t1 >= 0 and t1 <= time_step:
                t = t1
            elif t2 >= 0 and t2 <= time_step:
                t = t2
            else:
                # Collision occurs outside this time step
                return None
        
        # Calculate collision position and normal
        col_x1 = p1.x + p1.vx * t
        col_y1 = p1.y + p1.vy * t
        col_x2 = p2.x + p2.vx * t  
        col_y2 = p2.y + p2.vy * t
        
        # Collision position (midpoint between particle centers)
        col_x = (col_x1 + col_x2) / 2.0
        col_y = (col_y1 + col_y2) / 2.0
        
        # Normal vector (from p2 to p1)
        normal_x = col_x1 - col_x2
        normal_y = col_y1 - col_y2
        normal_length = math.sqrt(normal_x**2 + normal_y**2)
        
        if normal_length > 1e-10:
            normal_x /= normal_length
            normal_y /= normal_length
        else:
            # Particles at same position - use arbitrary normal
            normal_x, normal_y = 1.0, 0.0
        
        # Calculate relative velocity magnitude
        rel_vx = p1.vx - p2.vx
        rel_vy = p1.vy - p2.vy
        rel_vel = math.sqrt(rel_vx**2 + rel_vy**2)
        
        # Distance between centers at collision
        distance = math.sqrt((col_x1 - col_x2)**2 + (col_y1 - col_y2)**2)
        
        return Collision(
            frame=frame,
            time_s=frame * time_step + t,  # Absolute time
            toi=t,  # Time of impact within frame
            particle1=p1.particle_id,
            particle2=p2.particle_id, 
            x=col_x,
            y=col_y,
            nx=normal_x,
            ny=normal_y,
            distance=distance,
            relative_velocity=rel_vel
        )
    
    def run_brute_force_validation(self, max_frames: int = 18000, time_step: float = 1.0 / 60.0):
        """
        Run complete brute force validation
        
        Args:
            max_frames: Maximum number of frames to process
            time_step: Time step size for simulation
        """
        
        print(f" STARTING BRUTE FORCE COLLISION VALIDATION")
        print(f"="*60)
        print(f"Max Frames: {max_frames}")
        print(f"Time Step: {time_step}")
        
        # Load particle states
        particle_states = self.load_particle_states(max_frames)
        
        if not particle_states:
            print(" No particle states loaded - cannot proceed")
            return
        
        # Get all available frames and process up to max_frames
        all_frames = sorted(particle_states.keys())
        
        # Force to process frames 1 through max_frames (inclusive)
        frames_to_process = [f for f in range(1, max_frames + 1) if f in particle_states]
        
        print(f" Available frames: {min(all_frames)} to {max(all_frames)} ({len(all_frames)} total)")
        print(f" Processing {len(frames_to_process)} frames (1 to {len(frames_to_process)})...")
        
        # Perform brute force collision detection
        all_brute_force_collisions = []
        
        start_time = time.time()
        
        for frame_idx, frame in enumerate(frames_to_process):
            particles = particle_states[frame]
            
            # Detect collisions for this frame
            collisions = self.detect_collisions_brute_force(particles, frame, time_step)
            all_brute_force_collisions.extend(collisions)
            
            # Progress reporting
            if (frame_idx + 1) % 10 == 0 or frame_idx < 10:
                elapsed = time.time() - start_time
                progress = (frame_idx + 1) / len(frames_to_process) * 100
                eta = elapsed / (frame_idx + 1) * len(frames_to_process) - elapsed
                
                print(f"   Frame {frame}: {len(collisions)} collisions detected "
                      f"({progress:.1f}% complete, ETA: {eta:.1f}s)")
        
        total_time = time.time() - start_time
        print(f" Brute force detection completed in {total_time:.2f}s")
        print(f" Total brute force collisions detected: {len(all_brute_force_collisions)}")
        
        # Save brute force results
        self.save_brute_force_results(all_brute_force_collisions)
        
        # Load algorithm results and compare
        self.compare_algorithm_accuracy(all_brute_force_collisions, max_frames)
    
    def save_brute_force_results(self, collisions: List[Collision]):
        """Save brute force collision results to CSV"""
        
        if not collisions:
            print("  No brute force collisions to save")
            return
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        bf_file = os.path.join(self.output_dir, f"brute_force_collisions_{timestamp}.csv")
        
        # Convert to DataFrame
        collision_data = []
        for collision in collisions:
            collision_data.append({
                'frame': collision.frame,
                'time_s': collision.time_s,
                'toi': collision.toi,
                'particle1': collision.particle1,
                'particle2': collision.particle2,
                'x': collision.x,
                'y': collision.y,
                'nx': collision.nx,
                'ny': collision.ny,
                'distance': collision.distance,
                'relative_velocity': collision.relative_velocity
            })
        
        bf_df = pd.DataFrame(collision_data)
        bf_df.to_csv(bf_file, index=False)
        
        print(f" Brute force results saved to: {bf_file}")
        return bf_file
    
    def load_algorithm_events(self, max_frames: int = None) -> Tuple[pd.DataFrame, pd.DataFrame]:
        """Load T-CCD and Swept AABB collision events"""
        
        print(" Loading algorithm collision events...")
        
        try:
            # Load T-CCD events
            tccd_events = pd.read_csv(self.events_tccd_file)
            if max_frames is not None:
                tccd_events = tccd_events[tccd_events['frame'] <= max_frames]
            print(f"    T-CCD events: {len(tccd_events)} collisions")
            
            # Load Swept AABB events  
            swept_events = pd.read_csv(self.events_swept_file)
            if max_frames is not None:
                swept_events = swept_events[swept_events['frame'] <= max_frames]
            print(f"    Swept AABB events: {len(swept_events)} collisions")
            
            return tccd_events, swept_events
            
        except Exception as e:
            print(f"  Error loading algorithm events: {e}")
            return pd.DataFrame(), pd.DataFrame()
    
    def compare_algorithm_accuracy(self, brute_force_collisions: List[Collision], max_frames: int):
        """Compare algorithm results against brute force ground truth"""
        
        print(f"\n ALGORITHM ACCURACY COMPARISON")
        print(f"="*50)
        
        # Load algorithm events
        tccd_events, swept_events = self.load_algorithm_events(max_frames)
        
        if tccd_events.empty and swept_events.empty:
            print(" No algorithm events loaded - cannot compare")
            return
        
        # Convert brute force to comparison format
        bf_ground_truth = self._create_collision_set(brute_force_collisions)
        
        print(f" Ground Truth (Brute Force): {len(bf_ground_truth)} unique collisions")
        
        # Compare T-CCD
        if not tccd_events.empty:
            tccd_accuracy = self._compare_algorithm_events(
                tccd_events, bf_ground_truth, "T-CCD"
            )
        else:
            tccd_accuracy = None
        
        # Compare Swept AABB
        if not swept_events.empty:
            swept_accuracy = self._compare_algorithm_events(
                swept_events, bf_ground_truth, "Swept AABB"  
            )
        else:
            swept_accuracy = None
        
        # Save comparison results
        self.save_accuracy_comparison(tccd_accuracy, swept_accuracy, len(bf_ground_truth))
    
    def _create_collision_set(self, collisions: List[Collision], 
                             frame_tolerance: int = 1) -> Set[Tuple]:
        """
        Create a set of collision signatures for comparison
        
        Each collision is represented as (frame_range, particle_pair)
        to allow for tolerances in matching
        """
        collision_set = set()
        
        for collision in collisions:
            # Create frame range with tolerance
            frame_min = collision.frame - frame_tolerance
            frame_max = collision.frame + frame_tolerance
            
            # Ensure consistent particle ordering (smaller ID first)
            p1, p2 = sorted([collision.particle1, collision.particle2])
            
            collision_signature = (frame_min, frame_max, p1, p2)
            collision_set.add(collision_signature)
        
        return collision_set
    
    def _compare_algorithm_events(self, algorithm_events: pd.DataFrame, 
                                ground_truth: Set[Tuple], algorithm_name: str) -> Dict:
        """Compare algorithm events against ground truth"""
        
        print(f"\n Analyzing {algorithm_name} Accuracy...")
        
        true_positives = 0
        false_positives = 0
        algorithm_collisions = []
        
        # Convert ground truth to efficient lookup structure
        # Create dict: {(p1, p2): [list of valid frame ranges]}
        gt_lookup = {}
        for frame_min, frame_max, p1, p2 in ground_truth:
            key = (p1, p2)
            if key not in gt_lookup:
                gt_lookup[key] = []
            gt_lookup[key].append((frame_min, frame_max))
        
        print(f"   Processing {len(algorithm_events)} algorithm events...")
        
        # Check each algorithm event (much faster with lookup)
        for idx, event in algorithm_events.iterrows():
            if idx % 1000 == 0:  # Progress indicator
                print(f"   Progress: {idx}/{len(algorithm_events)} events processed")
                
            frame = int(event['frame'])
            
            # Skip wall collisions (where j might be 'bottom', 'top', 'left', 'right')
            try:
                p1_id = int(event['i'])
                p2_id = int(event['j'])
                p1, p2 = sorted([p1_id, p2_id])
            except (ValueError, TypeError):
                # Skip wall collisions - they're not particle-particle collisions
                continue
                
            algorithm_collisions.append((frame, p1, p2))
            
            # Fast lookup: check if particle pair exists in ground truth
            is_match = False
            particle_pair = (p1, p2)
            if particle_pair in gt_lookup:
                # Check if frame falls within any valid range for this particle pair
                for frame_min, frame_max in gt_lookup[particle_pair]:
                    if frame_min <= frame <= frame_max:
                        is_match = True
                        break
            
            if is_match:
                true_positives += 1
            else:
                false_positives += 1
        
        # Calculate false negatives efficiently
        # Create lookup of detected particle pairs with their frames
        detected_lookup = {}
        for frame, p1, p2 in algorithm_collisions:
            key = (p1, p2)
            if key not in detected_lookup:
                detected_lookup[key] = []
            detected_lookup[key].append(frame)
        
        false_negatives = 0
        print(f"   Checking {len(ground_truth)} ground truth collisions for false negatives...")
        
        for gt_signature in ground_truth:
            gt_frame_min, gt_frame_max, gt_p1, gt_p2 = gt_signature
            particle_pair = (gt_p1, gt_p2)
            
            is_detected = False
            if particle_pair in detected_lookup:
                # Check if any detected frame overlaps with ground truth frame range
                for detected_frame in detected_lookup[particle_pair]:
                    if gt_frame_min <= detected_frame <= gt_frame_max:
                        is_detected = True
                        break
            
            if not is_detected:
                false_negatives += 1
        
        # Calculate metrics
        precision = true_positives / (true_positives + false_positives) if (true_positives + false_positives) > 0 else 0.0
        recall = true_positives / (len(ground_truth)) if len(ground_truth) > 0 else 0.0  
        f1_score = 2 * (precision * recall) / (precision + recall) if (precision + recall) > 0 else 0.0
        
        accuracy_results = {
            'algorithm': algorithm_name,
            'detected_events': len(algorithm_events),
            'true_positives': true_positives,
            'false_positives': false_positives,
            'false_negatives': false_negatives,
            'precision': precision,
            'recall': recall,
            'f1_score': f1_score,
            'ground_truth_total': len(ground_truth)
        }
        
        # Print results
        print(f"    {algorithm_name} Results:")
        print(f"      Detected Events: {len(algorithm_events)}")
        print(f"      True Positives:  {true_positives}")
        print(f"      False Positives: {false_positives}")
        print(f"      False Negatives: {false_negatives}")
        print(f"      Precision:       {precision:.4f}")
        print(f"      Recall:          {recall:.4f}")
        print(f"      F1-Score:        {f1_score:.4f}")
        
        return accuracy_results
    
    def save_accuracy_comparison(self, tccd_results: Dict, swept_results: Dict, ground_truth_total: int):
        """Save accuracy comparison results to CSV"""
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        results_file = os.path.join(self.output_dir, f"accuracy_comparison_{timestamp}.csv")
        
        # Prepare results data
        results_data = []
        
        if tccd_results:
            results_data.append(tccd_results)
        
        if swept_results:
            results_data.append(swept_results)
        
        if results_data:
            df = pd.DataFrame(results_data)
            df['timestamp'] = datetime.now().isoformat()
            df['validation_method'] = 'brute_force'
            df.to_csv(results_file, index=False)
            
            print(f"\n Accuracy comparison saved to: {results_file}")
            
            # Print summary
            print(f"\n ACCURACY VALIDATION SUMMARY")
            print(f"="*40)
            print(f"Ground Truth Collisions: {ground_truth_total}")
            
            if tccd_results and swept_results:
                tccd_f1 = tccd_results['f1_score']
                swept_f1 = swept_results['f1_score']
                
                print(f"T-CCD F1-Score:    {tccd_f1:.4f}")
                print(f"Swept AABB F1-Score: {swept_f1:.4f}")
                
                if tccd_f1 > swept_f1:
                    print(f" T-CCD shows superior accuracy ({tccd_f1:.4f} vs {swept_f1:.4f})")
                elif swept_f1 > tccd_f1:
                    print(f" Swept AABB shows superior accuracy ({swept_f1:.4f} vs {tccd_f1:.4f})")
                else:
                    print(f" Both algorithms show similar accuracy (~{tccd_f1:.4f})")
        
        return results_file

def main():
    """Main function to run brute force validation"""
    
    print(" BRUTE FORCE COLLISION DETECTION VALIDATOR")
    print("="*60)
    print("This tool will:")
    print("1. Load your exact particle states from CSV files")  
    print("2. Perform exhaustive collision detection (ground truth)")
    print("3. Compare T-CCD and Swept AABB accuracy against ground truth")
    print("4. Generate detailed accuracy metrics and save to CSV")
    print("="*60)
    
    # Configuration
    MAX_FRAMES = 18000 
    TIME_STEP = 1.0 / 60.0  
    
    print(f"  Configuration:")
    print(f"   Max Frames: {MAX_FRAMES}")
    print(f"   Time Step: {TIME_STEP}")
    print()
    
    # Initialize validator
    validator = BruteForceValidator()
    
    # Run validation
    try:
        validator.run_brute_force_validation(
            max_frames=MAX_FRAMES,
            time_step=TIME_STEP
        )
        
        print(f"\n BRUTE FORCE VALIDATION COMPLETED!")
        print(f" Check the '{validator.output_dir}' directory for:")
        print(f"   - Brute force collision results (CSV)")
        print(f"   - Algorithm accuracy comparison (CSV)")
        print(f"   - Detailed precision, recall, and F1-score metrics")
        
    except Exception as e:
        print(f"\n Validation failed: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()