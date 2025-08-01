#!/usr/bin/env python3
"""
Python Security Vulnerabilities Test File
Contains various security issues for AI analysis testing
"""

import os
import subprocess
import sqlite3
import pickle
import yaml

# Security Issue 1: SQL Injection vulnerability
def get_user_data(user_id):
    """Vulnerable to SQL injection"""
    conn = sqlite3.connect('users.db')
    cursor = conn.cursor()
    
    # CRITICAL: SQL injection vulnerability
    query = f"SELECT * FROM users WHERE id = {user_id}"
    cursor.execute(query)  # Line 18: Direct string interpolation
    
    return cursor.fetchall()

# Security Issue 2: Command injection vulnerability  
def process_file(filename):
    """Vulnerable to command injection"""
    # CRITICAL: Command injection via shell=True
    result = subprocess.run(f"cat {filename}", shell=True, capture_output=True)  # Line 25
    return result.stdout

# Security Issue 3: Pickle deserialization vulnerability
def load_user_settings(data):
    """Dangerous deserialization"""
    # CRITICAL: Arbitrary code execution via pickle
    settings = pickle.loads(data)  # Line 31
    return settings

# Security Issue 4: Path traversal vulnerability
def read_config_file(config_name):
    """Path traversal vulnerability"""
    # HIGH: No path validation
    config_path = f"/app/configs/{config_name}"  # Line 37
    with open(config_path, 'r') as f:
        return f.read()

# Security Issue 5: Hardcoded credentials
DATABASE_PASSWORD = "admin123"  # Line 42: Hardcoded password
API_SECRET = "sk-1234567890abcdef"  # Line 43: Hardcoded API key

def connect_to_database():
    """Using hardcoded credentials"""
    # CRITICAL: Hardcoded credentials in code
    return f"postgresql://admin:{DATABASE_PASSWORD}@localhost/mydb"

# Security Issue 6: YAML deserialization vulnerability
def parse_config(yaml_content):
    """Unsafe YAML parsing"""
    # HIGH: yaml.load() can execute arbitrary code
    config = yaml.load(yaml_content)  # Line 53: Should use safe_load
    return config

# Security Issue 7: Weak random number generation
import random

def generate_session_token():
    """Weak random generation for security tokens"""
    # MEDIUM: Using predictable random for security
    token = ''.join([str(random.randint(0, 9)) for _ in range(32)])  # Line 61
    return token

# Security Issue 8: Information disclosure
def debug_user_info(user):
    """Information leakage in logs"""
    # MEDIUM: Sensitive data in logs
    print(f"User login: {user['username']}, Password: {user['password']}")  # Line 67
    print(f"Credit Card: {user['cc_number']}")  # Line 68

# Security Issue 9: Unsafe file operations
def save_uploaded_file(filename, content):
    """Unsafe file operations"""
    # HIGH: No file type validation
    filepath = f"/uploads/{filename}"  # Line 74
    with open(filepath, 'wb') as f:
        f.write(content)

# Performance and Code Quality Issues
def inefficient_operations():
    """Various performance issues"""
    # MEDIUM: Inefficient list operations
    large_list = []
    for i in range(100000):
        large_list.append(i)  # Line 83: Should use list comprehension
    
    # LOW: Unused variables
    unused_variable = "This is never used"  # Line 86
    another_unused = 42  # Line 87
    
    return large_list
