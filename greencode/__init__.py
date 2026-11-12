"""
GreenCode Profiler (GCP)
Universal Process-Level Energy Profiler & Green-Maintainability Framework.
Language-Agnostic: Supports PHP, Node.js, Python, Go, and compiled backend services.
"""

__version__ = "1.1.1"
__author__ = "GreenCode Team"

from .engine import ProcessEnergyEngine
from .attributor import ModuleAttributor
from .detector import RuntimeDetector
from .correlator import GreenCorrelator
from .analyzer import CodeQualityAnalyzer
from .reporter import ReportGenerator

__all__ = [
    "ProcessEnergyEngine",
    "ModuleAttributor",
    "RuntimeDetector",
    "GreenCorrelator",
    "CodeQualityAnalyzer",
    "ReportGenerator",
]
