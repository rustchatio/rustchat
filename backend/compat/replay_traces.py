#!/usr/bin/env python3
"""
Replay mitmproxy traces against RustChat and compare responses.

Usage:
    python3 replay_traces.py \
        --trace traces/mobile_session.mitm \
        --target http://localhost:8080 \
        --output diff_report.json \
        --ignore-fields "create_at,update_at,id,request_id"
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any
from urllib.parse import urljoin

try:
    import requests
    from mitmproxy.io import FlowReader
except ImportError:
    print("Install dependencies: pip install mitmproxy requests")
    sys.exit(1)


def parse_args():
    parser = argparse.ArgumentParser(description="Replay mitmproxy traces")
    parser.add_argument("--trace", required=True, help="Path to .mitm trace file")
    parser.add_argument("--target", required=True, help="Target server URL")
    parser.add_argument("--output", default="diff_report.json", help="Output file")
    parser.add_argument("--ignore-fields", default="", help="Comma-separated fields to ignore")
    parser.add_argument("--filter-path", default="/api/v4", help="Only replay paths matching this prefix")
    return parser.parse_args()


def normalize_response(data: Any, ignore_fields: set) -> Any:
    """Remove ignored fields for comparison."""
    if isinstance(data, dict):
        return {
            k: normalize_response(v, ignore_fields)
            for k, v in data.items()
            if k not in ignore_fields
        }
    elif isinstance(data, list):
        return [normalize_response(item, ignore_fields) for item in data]
    return data


def compare_responses(expected: Any, actual: Any) -> list[str]:
    """Compare two responses and return list of differences."""
    diffs = []
    
    if type(expected) != type(actual):
        return [f"Type mismatch: expected {type(expected).__name__}, got {type(actual).__name__}"]
    
    if isinstance(expected, dict):
        for key in set(expected.keys()) | set(actual.keys()):
            if key not in expected:
                diffs.append(f"Extra key in actual: {key}")
            elif key not in actual:
                diffs.append(f"Missing key in actual: {key}")
            else:
                nested = compare_responses(expected[key], actual[key])
                diffs.extend([f"{key}.{d}" for d in nested])
    elif isinstance(expected, list):
        if len(expected) != len(actual):
            diffs.append(f"Array length mismatch: expected {len(expected)}, got {len(actual)}")
        for i, (e, a) in enumerate(zip(expected, actual)):
            nested = compare_responses(e, a)
            diffs.extend([f"[{i}].{d}" for d in nested])
    elif expected != actual:
        diffs.append(f"Value mismatch: expected {expected!r}, got {actual!r}")
    
    return diffs


def replay_flow(flow, target_url: str, session: requests.Session) -> dict:
    """Replay a single flow and return comparison result."""
    request = flow.request
    
    # Build target URL
    path = request.path
    url = urljoin(target_url, path)
    
    # Prepare headers (remove host, use auth token)
    headers = {k: v for k, v in request.headers.items() 
               if k.lower() not in ("host", "content-length")}
    
    # Make request
    try:
        response = session.request(
            method=request.method,
            url=url,
            headers=headers,
            data=request.content if request.content else None,
            timeout=10,
        )
        actual_body = response.json() if response.content else None
    except Exception as e:
        return {
            "path": path,
            "method": request.method,
            "error": str(e),
            "success": False,
        }
    
    # Parse original response
    original = flow.response
    try:
        expected_body = json.loads(original.content) if original.content else None
    except json.JSONDecodeError:
        expected_body = None
    
    return {
        "path": path,
        "method": request.method,
        "expected_status": original.status_code,
        "actual_status": response.status_code,
        "status_match": original.status_code == response.status_code,
        "success": True,
        "expected_body": expected_body,
        "actual_body": actual_body,
    }


def main():
    args = parse_args()
    ignore_fields = set(f.strip() for f in args.ignore_fields.split(",") if f.strip())
    
    trace_path = Path(args.trace)
    if not trace_path.exists():
        print(f"Trace file not found: {trace_path}")
        sys.exit(1)
    
    results = []
    session = requests.Session()
    
    with open(trace_path, "rb") as f:
        reader = FlowReader(f)
        for flow in reader.stream():
            if not flow.request.path.startswith(args.filter_path):
                continue
            
            result = replay_flow(flow, args.target, session)
            
            # Compare normalized responses
            if result.get("success") and result.get("expected_body") and result.get("actual_body"):
                expected_norm = normalize_response(result["expected_body"], ignore_fields)
                actual_norm = normalize_response(result["actual_body"], ignore_fields)
                result["diffs"] = compare_responses(expected_norm, actual_norm)
                result["compatible"] = len(result["diffs"]) == 0 and result["status_match"]
            else:
                result["compatible"] = result.get("status_match", False)
            
            results.append(result)
            print(f"{'✓' if result.get('compatible') else '✗'} {result['method']} {result['path']}")
    
    # Write report
    with open(args.output, "w") as f:
        json.dump({
            "total": len(results),
            "compatible": sum(1 for r in results if r.get("compatible")),
            "incompatible": sum(1 for r in results if not r.get("compatible")),
            "results": results,
        }, f, indent=2)
    
    print(f"\nReport written to {args.output}")
    compatible = sum(1 for r in results if r.get("compatible"))
    print(f"Compatible: {compatible}/{len(results)}")


if __name__ == "__main__":
    main()
