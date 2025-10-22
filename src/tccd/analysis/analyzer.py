"""Accuracy analysis utilities for collision detection algorithms."""

from typing import Dict, List, Set, Tuple

import pandas as pd
from ..models import Collision


class AccuracyAnalyzer:
    """Analyzes and compares algorithm accuracy against ground truth."""
    
    @staticmethod
    def create_collision_signature_set(collisions: List[Collision], 
                                     frame_tolerance: int = 1) -> Set[Tuple]:
        """Create a set of collision signatures for comparison."""
        collision_set = set()
        
        for collision in collisions:
            # Create frame range with tolerance
            frame_min = collision.frame - frame_tolerance
            frame_max = collision.frame + frame_tolerance
            
            # Ensure consistent particle ordering
            p1, p2 = collision.particle_pair
            
            collision_signature = (frame_min, frame_max, p1, p2)
            collision_set.add(collision_signature)
        
        return collision_set
    
    @staticmethod
    def compare_algorithm_events(algorithm_events: pd.DataFrame, 
                               ground_truth: Set[Tuple], 
                               algorithm_name: str) -> Dict:
        """Compare algorithm events against ground truth with optimized lookup."""
        print(f" Analyzing {algorithm_name} accuracy...")
        
        true_positives = 0
        false_positives = 0
        algorithm_collisions = []
        
        # Create efficient lookup structure
        gt_lookup = AccuracyAnalyzer._create_ground_truth_lookup(ground_truth)
        
        print(f"   Processing {len(algorithm_events)} algorithm events...")
        
        # Process algorithm events
        for idx, event in algorithm_events.iterrows():
            if idx % 1000 == 0:
                print(f"   Progress: {idx}/{len(algorithm_events)} events processed")
                
            frame = int(event['frame'])
            
            # Skip wall collisions
            try:
                p1_id = int(event['i'])
                p2_id = int(event['j'])
                p1, p2 = sorted([p1_id, p2_id])
            except (ValueError, TypeError):
                continue
                
            algorithm_collisions.append((frame, p1, p2))
            
            # Fast lookup for ground truth match
            is_match = AccuracyAnalyzer._check_ground_truth_match(
                frame, (p1, p2), gt_lookup
            )
            
            if is_match:
                true_positives += 1
            else:
                false_positives += 1
        
        # Calculate false negatives
        false_negatives = AccuracyAnalyzer._calculate_false_negatives(
            algorithm_collisions, ground_truth
        )
        
        # Calculate metrics
        precision = true_positives / (true_positives + false_positives) if (true_positives + false_positives) > 0 else 0.0
        recall = true_positives / len(ground_truth) if len(ground_truth) > 0 else 0.0
        f1_score = 2 * (precision * recall) / (precision + recall) if (precision + recall) > 0 else 0.0
        
        results = {
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
        
        AccuracyAnalyzer._print_results(results)
        return results
    
    @staticmethod
    def _create_ground_truth_lookup(ground_truth: Set[Tuple]) -> Dict:
        """Create efficient lookup structure for ground truth."""
        gt_lookup = {}
        for frame_min, frame_max, p1, p2 in ground_truth:
            key = (p1, p2)
            if key not in gt_lookup:
                gt_lookup[key] = []
            gt_lookup[key].append((frame_min, frame_max))
        return gt_lookup
    
    @staticmethod
    def _check_ground_truth_match(frame: int, particle_pair: Tuple[int, int], 
                                gt_lookup: Dict) -> bool:
        """Check if detected collision matches ground truth."""
        if particle_pair not in gt_lookup:
            return False
        
        for frame_min, frame_max in gt_lookup[particle_pair]:
            if frame_min <= frame <= frame_max:
                return True
        return False
    
    @staticmethod
    def _calculate_false_negatives(algorithm_collisions: List[Tuple], 
                                 ground_truth: Set[Tuple]) -> int:
        """Calculate false negatives efficiently."""
        detected_lookup = {}
        for frame, p1, p2 in algorithm_collisions:
            key = (p1, p2)
            if key not in detected_lookup:
                detected_lookup[key] = []
            detected_lookup[key].append(frame)
        
        false_negatives = 0
        for gt_frame_min, gt_frame_max, gt_p1, gt_p2 in ground_truth:
            particle_pair = (gt_p1, gt_p2)
            
            is_detected = False
            if particle_pair in detected_lookup:
                for detected_frame in detected_lookup[particle_pair]:
                    if gt_frame_min <= detected_frame <= gt_frame_max:
                        is_detected = True
                        break
            
            if not is_detected:
                false_negatives += 1
        
        return false_negatives
    
    @staticmethod
    def _print_results(results: Dict):
        """Print formatted accuracy results."""
        print(f"    📊 {results['algorithm']} Results:")
        print(f"      Detected Events: {results['detected_events']:,}")
        print(f"      True Positives:  {results['true_positives']:,}")
        print(f"      False Positives: {results['false_positives']:,}")
        print(f"      False Negatives: {results['false_negatives']:,}")
        print(f"      Precision:       {results['precision']:.4f}")
        print(f"      Recall:          {results['recall']:.4f}")
        print(f"      F1-Score:        {results['f1_score']:.4f}")