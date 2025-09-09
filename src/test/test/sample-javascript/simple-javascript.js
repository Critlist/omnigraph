// Simple JavaScript test file for parser validation

// Import statements
import { writeFile } from "fs/promises";

// Default import

// Constants and variables
const API_BASE_URL = "https://api.example.com";


// Enum-like object
const UserStatus = {
	ACTIVE: "active",
	INACTIVE: "inactive",
	PENDING: "pending",
};

// Simple class
class Logger {
	constructor(level = "info") {
		this.level = level;
		this.logs = [];
	}

	static getInstance() {
		if (!Logger.instance) {
			Logger.instance = new Logger();
		}
		return Logger.instance;
	}

	log(message, level = "info") {
		const timestamp = new Date().toISOString();
		const logEntry = { timestamp, message, level };
		this.logs.push(logEntry);
		console.log(`[${timestamp}] ${level.toUpperCase()}: ${message}`);
	}

	error(message) {
		this.log(message, "error");
	}

	warn(message) {
		this.log(message, "warn");
	}

	getLogs() {
		return [...this.logs];
	}
}

// Extended class
class FileLogger extends Logger {
	constructor(level, filePath) {
		super(level);
		this.filePath = filePath;
	}

	async log(message, level = "info") {
		super.log(message, level);

		try {
			const logLine = `[${new Date().toISOString()}] ${level.toUpperCase()}: ${message}\n`;
			await writeFile(this.filePath, logLine, { flag: "a" });
		} catch (error) {
			console.error("Failed to write to log file:", error);
		}
	}
}

// Traditional function
function calculateSum(a, b) {
	return a + b;
}

// Arrow functions
const multiply = (a, b) => a * b;

const processData = async (data) => {
	try {
		const processed = data.map((item) => ({
			...item,
			processed: true,
			timestamp: Date.now(),
		}));

		return processed;
	} catch (error) {
		throw new Error(`Failed to process data: ${error.message}`);
	}
};

// Function with default parameters and destructuring
const createUser = ({
	name,
	email,
	age = 18,
	status = UserStatus.PENDING,
	...otherProps
}) => {
	return {
		id: Math.random().toString(36).substr(2, 9),
		name,
		email,
		age,
		status,
		createdAt: new Date(),
		...otherProps,
	};
};

// Generator function
function* fibonacci(limit) {
	let a = 0,
		b = 1;
	while (a < limit) {
		yield a;
		[a, b] = [b, a + b];
	}
}

// Async generator
async function* fetchDataPages(url) {
	let page = 1;
	let hasMore = true;

	while (hasMore) {
		try {
			const response = await fetch(`${url}?page=${page}`);
			const data = await response.json();

			yield data.items;

			hasMore = data.hasMore;
			page++;
		} catch (error) {
			throw new Error(`Failed to fetch page ${page}: ${error.message}`);
		}
	}
}

// Complex object with methods
const ApiClient = {
	baseURL: API_BASE_URL,
	headers: {
		"Content-Type": "application/json",
		Accept: "application/json",
	},

	async get(endpoint) {
		const url = `${this.baseURL}${endpoint}`;
		const response = await fetch(url, {
			method: "GET",
			headers: this.headers,
		});

		if (!response.ok) {
			throw new Error(`HTTP ${response.status}: ${response.statusText}`);
		}

		return response.json();
	},

	async post(endpoint, data) {
		const url = `${this.baseURL}${endpoint}`;
		const response = await fetch(url, {
			method: "POST",
			headers: this.headers,
			body: JSON.stringify(data),
		});

		if (!response.ok) {
			throw new Error(`HTTP ${response.status}: ${response.statusText}`);
		}

		return response.json();
	},
};

// Higher-order function
const withLogging = (func) => {
	return async (...args) => {
		const logger = Logger.getInstance();
		logger.log(`Calling function: ${func.name || "anonymous"}`);

		try {
			const result = await func(...args);
			logger.log(
				`Function completed successfully: ${func.name || "anonymous"}`,
			);
			return result;
		} catch (error) {
			logger.error(
				`Function failed: ${func.name || "anonymous"} - ${error.message}`,
			);
			throw error;
		}
	};
};

// Function with complex control flow
function processUserData(users) {
	const results = [];

	for (const user of users) {
		if (!user || typeof user !== "object") {
			continue;
		}

		const processedUser = { ...user };

		// Switch statement
		switch (user.status) {
			case UserStatus.ACTIVE:
				processedUser.isActive = true;
				break;
			case UserStatus.INACTIVE:
				processedUser.isActive = false;
				processedUser.deactivatedAt = new Date();
				break;
			case UserStatus.PENDING:
				processedUser.isPending = true;
				break;
			default:
				processedUser.hasUnknownStatus = true;
		}

		// Nested conditions
		if (user.age) {
			if (user.age < 18) {
				processedUser.isMinor = true;
			} else if (user.age >= 65) {
				processedUser.isSenior = true;
			} else {
				processedUser.isAdult = true;
			}
		}

		// Try-catch for error handling
		try {
			processedUser.emailDomain = user.email.split("@")[1];
		} catch {
			processedUser.emailDomain = null;
		}

		results.push(processedUser);
	}

	return results;
}

// Closure example
const createCounter = (initialValue = 0) => {
	let count = initialValue;

	return {
		increment: () => ++count,
		decrement: () => --count,
		getValue: () => count,
		reset: () => {
			count = initialValue;
		},
	};
};

// Prototype extension
Array.prototype.groupBy = function (keyFunc) {
	return this.reduce((groups, item) => {
		const key = keyFunc(item);
		if (!groups[key]) {
			groups[key] = [];
		}
		groups[key].push(item);
		return groups;
	}, {});
};

// Module exports
export default ApiClient;
export {
	Logger,
	FileLogger,
	calculateSum,
	multiply,
	processData,
	createUser,
	fibonacci,
	fetchDataPages,
	withLogging,
	processUserData,
	createCounter,
	UserStatus,
};

// CommonJS style export (mixed with ES6)
module.exports = {
	ApiClient,
	Logger,
	processUserData,
};
