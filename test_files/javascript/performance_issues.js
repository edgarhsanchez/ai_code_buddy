// JavaScript Performance Issues Test File
function performanceIssues() {
    // Performance Issue 1: N+1 Query Problem
    const users = fetchAllUsers();
    for (let user of users) {
        const posts = fetchUserPosts(user.id); // Inefficient database queries
        console.log(posts);
    }
    
    // Performance Issue 2: Memory Leak - Event listeners not removed
    function addEventListeners() {
        for (let i = 0; i < 10000; i++) {
            const element = document.createElement('div');
            element.addEventListener('click', () => {
                console.log('Clicked:', i); // Closure retains all variables
            });
            document.body.appendChild(element);
        }
    }
    
    // Performance Issue 3: Blocking main thread
    function heavyComputation() {
        let result = 0;
        for (let i = 0; i < 10000000; i++) {
            result += Math.sqrt(i) * Math.random();
        }
        return result;
    }
    
    // Performance Issue 4: Inefficient array operations
    let largeArray = new Array(100000).fill(0);
    largeArray.forEach((item, index) => {
        largeArray[index] = item + 1; // Modifying array while iterating
        document.querySelector('.status').textContent = `Processing ${index}`; // DOM update in loop
    });
    
    // Performance Issue 5: Multiple forced reflows
    const elements = document.querySelectorAll('.item');
    elements.forEach(el => {
        el.style.width = '100px';
        el.offsetHeight; // Forces reflow
        el.style.height = '100px';
        el.offsetWidth; // Forces another reflow
    });
}

// Memory management issues
class DataProcessor {
    constructor() {
        this.cache = new Map();
        this.timer = setInterval(() => {
            this.processData();
        }, 100);
        // Missing cleanup in destructor
    }
    
    processData() {
        // Cache grows indefinitely
        this.cache.set(Date.now(), new Array(1000).fill(Math.random()));
    }
}
