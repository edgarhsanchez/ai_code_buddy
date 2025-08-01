// OWASP Top 10 Vulnerability Test File
// This file contains examples of common OWASP security vulnerabilities for testing the AI Code Buddy analysis

// ============================================================================
// OWASP A01:2021 – Broken Access Control
// ============================================================================

// Insecure Direct Object Reference
function getUserData(userId) {
    // VULNERABLE: No authorization check
    return database.query(`SELECT * FROM users WHERE id = ${userId}`);
}

// Missing Authorization
app.get('/admin/users', (req, res) => {
    // VULNERABLE: No admin role verification
    const users = database.getAllUsers();
    res.json(users);
});

// Path Traversal
app.get('/files/:filename', (req, res) => {
    // VULNERABLE: No path validation
    const filename = req.params.filename;
    const filepath = `/uploads/${filename}`;
    res.sendFile(filepath);
});

// ============================================================================
// OWASP A02:2021 – Cryptographic Failures
// ============================================================================

// Hardcoded Secrets
const API_KEY = "sk-1234567890abcdef";
const DATABASE_PASSWORD = "admin123";
const JWT_SECRET = "mysecretkey";

// Weak Encryption
const crypto = require('crypto');
function weakEncrypt(data) {
    // VULNERABLE: MD5 is cryptographically broken
    return crypto.createHash('md5').update(data).digest('hex');
}

// Insecure Storage
function storePassword(password) {
    // VULNERABLE: Storing password in plain text
    localStorage.setItem('userPassword', password);
}

// ============================================================================
// OWASP A03:2021 – Injection
// ============================================================================

// SQL Injection
function loginUser(username, password) {
    // VULNERABLE: Direct string concatenation
    const query = `SELECT * FROM users WHERE username = '${username}' AND password = '${password}'`;
    return database.query(query);
}

// NoSQL Injection
function findUser(req) {
    // VULNERABLE: Direct object injection
    const userQuery = req.body.user;
    return db.collection('users').findOne(userQuery);
}

// Command Injection
const { exec } = require('child_process');
function processFile(filename) {
    // VULNERABLE: Unsanitized input in command
    exec(`convert ${filename} output.jpg`, (error, stdout, stderr) => {
        console.log(stdout);
    });
}

// LDAP Injection
function authenticateLDAP(username) {
    // VULNERABLE: Unsanitized LDAP query
    const filter = `(uid=${username})`;
    return ldapClient.search('ou=people,dc=example,dc=com', { filter });
}

// ============================================================================
// OWASP A04:2021 – Insecure Design
// ============================================================================

// Insufficient Rate Limiting
app.post('/api/login', (req, res) => {
    // VULNERABLE: No rate limiting on login attempts
    const { username, password } = req.body;
    if (authenticateUser(username, password)) {
        res.json({ success: true });
    } else {
        res.status(401).json({ error: 'Invalid credentials' });
    }
});

// Missing Security Headers
app.use((req, res, next) => {
    // VULNERABLE: Missing security headers
    next();
});

// ============================================================================
// OWASP A05:2021 – Security Misconfiguration
// ============================================================================

// Debug Mode in Production
const DEBUG = true;
if (DEBUG) {
    console.log("Sensitive debug info: ", process.env);
}

// Default Credentials
const defaultAdmin = {
    username: 'admin',
    password: 'admin'
};

// Excessive Permissions
app.use(cors({
    // VULNERABLE: Overly permissive CORS
    origin: '*',
    credentials: true
}));

// ============================================================================
// OWASP A06:2021 – Vulnerable and Outdated Components
// ============================================================================

// Using Vulnerable Libraries (package.json would show this)
const express = require('express'); // Assume old version with vulnerabilities
const lodash = require('lodash'); // Assume vulnerable version

// ============================================================================
// OWASP A07:2021 – Identification and Authentication Failures
// ============================================================================

// Weak Password Policy
function isValidPassword(password) {
    // VULNERABLE: Weak password requirements
    return password.length >= 4;
}

// Session Fixation
app.post('/login', (req, res) => {
    if (authenticateUser(req.body.username, req.body.password)) {
        // VULNERABLE: Not regenerating session ID after login
        req.session.user = req.body.username;
        res.json({ success: true });
    }
});

// Missing Multi-Factor Authentication
function sensitiveOperation(userId) {
    // VULNERABLE: No MFA for sensitive operations
    if (req.session.user) {
        return performSensitiveAction(userId);
    }
}

// ============================================================================
// OWASP A08:2021 – Software and Data Integrity Failures
// ============================================================================

// Insecure Deserialization
function deserializeUserData(serializedData) {
    // VULNERABLE: Using eval for deserialization
    return eval('(' + serializedData + ')');
}

// Missing Integrity Checks
function downloadAndExecute(url) {
    // VULNERABLE: No integrity verification
    fetch(url).then(response => response.text()).then(code => {
        eval(code);
    });
}

// ============================================================================
// OWASP A09:2021 – Security Logging and Monitoring Failures
// ============================================================================

// Insufficient Logging
function deleteUser(userId) {
    // VULNERABLE: No audit logging for sensitive operations
    database.query(`DELETE FROM users WHERE id = ${userId}`);
}

// Logging Sensitive Data
function logUserActivity(user, action) {
    // VULNERABLE: Logging sensitive information
    console.log(`User: ${user.email}, Password: ${user.password}, Action: ${action}`);
}

// ============================================================================
// OWASP A10:2021 – Server-Side Request Forgery (SSRF)
// ============================================================================

// Unvalidated URL Fetching
app.get('/fetch', async (req, res) => {
    // VULNERABLE: No URL validation
    const url = req.query.url;
    const response = await fetch(url);
    const data = await response.text();
    res.send(data);
});

// Internal Service Access
function callInternalAPI(endpoint) {
    // VULNERABLE: Allowing access to internal services
    const internalUrl = `http://internal-api/${endpoint}`;
    return fetch(internalUrl);
}

// ============================================================================
// Additional Common Vulnerabilities
// ============================================================================

// Cross-Site Scripting (XSS)
function displayUserComment(comment) {
    // VULNERABLE: Direct innerHTML without sanitization
    document.getElementById('comments').innerHTML = comment;
}

// DOM-based XSS
function processUrlParameter() {
    // VULNERABLE: Direct use of URL parameters in DOM
    const userInput = window.location.search.substring(1);
    document.body.innerHTML = `<h1>Welcome ${userInput}</h1>`;
}

// Open Redirect
function redirectUser(url) {
    // VULNERABLE: Unvalidated redirect
    window.location = url;
}

// Insecure Cookie Configuration
app.use(session({
    secret: 'keyboard cat',
    cookie: {
        // VULNERABLE: Missing security flags
        secure: false,
        httpOnly: false,
        sameSite: false
    }
}));

// Information Disclosure
app.use((err, req, res, next) => {
    // VULNERABLE: Exposing stack traces
    res.status(500).json({
        error: err.message,
        stack: err.stack,
        details: process.env
    });
});

// Race Condition
let balance = 1000;
function withdraw(amount) {
    // VULNERABLE: Race condition in financial operation
    if (balance >= amount) {
        setTimeout(() => {
            balance -= amount;
        }, 100);
        return true;
    }
    return false;
}

// Time-based Attack
function comparePasswords(provided, stored) {
    // VULNERABLE: Time-based side channel attack
    for (let i = 0; i < Math.max(provided.length, stored.length); i++) {
        if (provided[i] !== stored[i]) {
            return false;
        }
    }
    return provided.length === stored.length;
}

// Prototype Pollution
function merge(target, source) {
    // VULNERABLE: Prototype pollution
    for (let key in source) {
        target[key] = source[key];
    }
    return target;
}

// Regular Expression Denial of Service (ReDoS)
function validateEmail(email) {
    // VULNERABLE: Catastrophic backtracking
    const emailRegex = /^([a-zA-Z0-9_\-\.]+)@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.)|(([a-zA-Z0-9\-]+\.)+))([a-zA-Z]{2,4}|[0-9]{1,3})(\]?)$/;
    return emailRegex.test(email);
}

module.exports = {
    getUserData,
    weakEncrypt,
    loginUser,
    processFile,
    deserializeUserData,
    displayUserComment,
    redirectUser,
    withdraw,
    comparePasswords,
    merge,
    validateEmail
};
