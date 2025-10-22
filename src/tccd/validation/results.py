"""Results management utilities for T-CCD validation framework."""

import os
from datetime import datetime
from typing import Dict, List, Optional

import pandas as pd

from ..config import output_dir
from ..models import Collision


class ResultsManager:
    """Handles saving and managing validation results."""
    
    def __init__(self, output_dir: str = output_dir):
        self.output_dir = output_dir
        os.makedirs(output_dir, exist_ok=True)
    
    def save_brute_force_results(self, collisions: List[Collision]) -> Optional[str]:
        """Save brute force collision results to CSV."""
        if not collisions:
            print(" No brute force collisions to save")
            return None
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"brute_force_collisions_{timestamp}.csv"
        filepath = os.path.join(self.output_dir, filename)
        
        # Convert to DataFrame
        collision_data = [
            {
                'frame': c.frame,
                'time_s': c.time_s,
                'toi': c.toi,
                'particle1': c.particle1,
                'particle2': c.particle2,
                'x': c.x,
                'y': c.y,
                'nx': c.nx,
                'ny': c.ny,
                'distance': c.distance,
                'relative_velocity': c.relative_velocity
            }
            for c in collisions
        ]
        
        df = pd.DataFrame(collision_data)
        df.to_csv(filepath, index=False)
        
        print(f" Brute force results saved: {filepath}")
        return filepath
    
    def save_accuracy_comparison(self, tccd_results: Optional[Dict], swept_results: Optional[Dict], 
                               ground_truth_total: int) -> Optional[str]:
        """Save accuracy comparison results to CSV."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"accuracy_comparison_{timestamp}.csv"
        filepath = os.path.join(self.output_dir, filename)
        
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
            df.to_csv(filepath, index=False)
            
            print(f" Accuracy comparison saved: {filepath}")
            self._print_summary(tccd_results, swept_results, ground_truth_total)
        
        return filepath
    
    def _print_summary(self, tccd_results: Optional[Dict], swept_results: Optional[Dict], 
                      ground_truth_total: int):
        """Print validation summary."""
        print(f"\n ACCURACY VALIDATION SUMMARY")
        print(f"=" * 40)
        print(f"Ground Truth Collisions: {ground_truth_total:,}")
        
        if tccd_results and swept_results:
            tccd_f1 = tccd_results['f1_score']
            swept_f1 = swept_results['f1_score']
            
            print(f"T-CCD F1-Score:      {tccd_f1:.4f}")
            print(f"Swept AABB F1-Score: {swept_f1:.4f}")
            
            if tccd_f1 > swept_f1:
                print(f" T-CCD shows superior accuracy ({tccd_f1:.4f} vs {swept_f1:.4f})")
            elif swept_f1 > tccd_f1:
                print(f" Swept AABB shows superior accuracy ({swept_f1:.4f} vs {tccd_f1:.4f})")
            else:
                print(f" Both algorithms show similar accuracy (~{tccd_f1:.4f})")