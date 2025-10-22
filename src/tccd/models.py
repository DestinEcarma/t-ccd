"""Data structures for T-CCD collision detection."""

from dataclasses import dataclass
from typing import Tuple


@dataclass(frozen=True)
class Particle:
    """Immutable particle state at a specific frame."""
    particle_id: int
    frame: int
    x: float
    y: float
    vx: float
    vy: float
    radius: float
    mass: float = 1.0

    def position_at_time(self, t: float) -> Tuple[float, float]:
        """Calculate particle position at time t."""
        return (self.x + self.vx * t, self.y + self.vy * t)


@dataclass(frozen=True)
class Collision:
    """Immutable collision event detected by brute force analysis."""
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

    @property
    def particle_pair(self) -> Tuple[int, int]:
        """Return sorted particle pair for consistent comparison."""
        return tuple(sorted([self.particle1, self.particle2]))