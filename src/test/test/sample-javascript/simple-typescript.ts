// TypeScript test file for parser validation

// Type aliases
type ID = string | number;
type Status = "active" | "inactive" | "pending";
type EventHandler<T> = (event: T) => void;

// String literal types and mapped types
type HttpMethod = "GET" | "POST" | "PUT" | "DELETE";
type ApiEndpoints = {
	[K in HttpMethod]: string[];
};

// Utility types
type Partial<T> = {
	[P in keyof T]?: T[P];
};

type Optional<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>> &
	Partial<Pick<T, K>>;

// Interfaces
interface BaseEntity {
	readonly id: ID;
	createdAt: Date;
	updatedAt?: Date;
}

interface User extends BaseEntity {
	name: string;
	email: string;
	age: number;
	status: Status;
	roles: Role[];
	profile?: UserProfile;
}

interface UserProfile {
	bio?: string;
	avatar?: string;
	preferences: UserPreferences;
}

interface UserPreferences {
	theme: "light" | "dark";
	notifications: boolean;
	language: string;
}

interface Role {
	id: string;
	name: string;
	permissions: Permission[];
}

interface Permission {
	resource: string;
	actions: string[];
}

// Generic interfaces
interface Repository<T extends BaseEntity> {
	findById(id: ID): Promise<T | null>;
	findAll(): Promise<T[]>;
	create(entity: Omit<T, "id" | "createdAt">): Promise<T>;
	update(id: ID, updates: Partial<T>): Promise<T>;
	delete(id: ID): Promise<boolean>;
}

interface ApiResponse<T> {
	data: T;
	success: boolean;
	message?: string;
	errors?: string[];
}

interface PaginatedResponse<T> extends ApiResponse<T[]> {
	pagination: {
		page: number;
		limit: number;
		total: number;
		pages: number;
	};
}

// Enums
enum UserRole {
	ADMIN = "admin",
	MODERATOR = "moderator",
	USER = "user",
	GUEST = "guest",
}

enum HttpStatusCode {
	OK = 200,
	CREATED = 201,
	BAD_REQUEST = 400,
	UNAUTHORIZED = 401,
	FORBIDDEN = 403,
	NOT_FOUND = 404,
	INTERNAL_SERVER_ERROR = 500,
}

const enum Direction {
	Up = "UP",
	Down = "DOWN",
	Left = "LEFT",
	Right = "RIGHT",
}

// Abstract class
abstract class BaseService<T extends BaseEntity> {
	protected abstract repository: Repository<T>;

	abstract validate(entity: T): boolean;

	async findById(id: ID): Promise<T | null> {
		return this.repository.findById(id);
	}

	async create(entity: Omit<T, "id" | "createdAt">): Promise<T> {
		const newEntity = {
			...entity,
			id: this.generateId(),
			createdAt: new Date(),
		} as T;

		if (!this.validate(newEntity)) {
			throw new Error("Validation failed");
		}

		return this.repository.create(entity);
	}

	protected generateId(): string {
		return Math.random().toString(36).substr(2, 9);
	}
}

// Generic class
class UserService extends BaseService<User> {
	protected repository: Repository<User>;

	constructor(repository: Repository<User>) {
		super();
		this.repository = repository;
	}

	validate(user: User): boolean {
		return (
			typeof user.name === "string" &&
			user.name.length > 0 &&
			typeof user.email === "string" &&
			user.email.includes("@") &&
			typeof user.age === "number" &&
			user.age >= 0
		);
	}

	async findByEmail(email: string): Promise<User | null> {
		const users = await this.repository.findAll();
		return users.find((user) => user.email === email) || null;
	}

	async updateStatus(id: ID, status: Status): Promise<User> {
		return this.repository.update(id, { status, updatedAt: new Date() });
	}

	hasRole(user: User, role: UserRole): boolean {
		return user.roles.some((r) => r.name === role);
	}

	hasPermission(user: User, resource: string, action: string): boolean {
		return user.roles.some((role) =>
			role.permissions.some(
				(permission) =>
					permission.resource === resource &&
					permission.actions.includes(action),
			),
		);
	}
}

// Generic functions
function identity<T>(arg: T): T {
	return arg;
}

function firstElement<T>(arr: T[]): T | undefined {
	return arr[0];
}

function mapArray<T, U>(arr: T[], mapper: (item: T) => U): U[] {
	return arr.map(mapper);
}

// Function overloads
function processValue(value: string): string;
function processValue(value: number): number;
function processValue(value: boolean): boolean;
function processValue(
	value: string | number | boolean,
): string | number | boolean {
	if (typeof value === "string") {
		return value.toUpperCase();
	} else if (typeof value === "number") {
		return value * 2;
	} else {
		return !value;
	}
}

// Conditional types
type NonNullable<T> = T extends null | undefined ? never : T;
type ReturnType<T> = T extends (...args: any[]) => infer R ? R : any;
type Parameters<T> = T extends (...args: infer P) => any ? P : never;

// Template literal types
type EventName<T extends string> = `on${Capitalize<T>}`;
type CSSProperty = `--${string}`;

// Mapped types
type ReadonlyUser = {
	readonly [K in keyof User]: User[K];
};

type PartialUser = {
	[K in keyof User]?: User[K];
};

type UserKeys = keyof User;
type UserValues = User[UserKeys];

// Index signatures
interface StringDictionary {
	[key: string]: string;
}



// Class with decorators (TypeScript experimental feature)
class ApiController {
	private users: User[] = [];

	// @Route('GET', '/users')
	getAllUsers(): User[] {
		return this.users;
	}

	// @Route('GET', '/users/:id')
	getUserById(id: ID): User | undefined {
		return this.users.find((user) => user.id === id);
	}

	// @Route('POST', '/users')
	// @ValidateBody(UserSchema)
	createUser(userData: Omit<User, "id" | "createdAt">): User {
		const user: User = {
			...userData,
			id: this.generateId(),
			createdAt: new Date(),
		};

		this.users.push(user);
		return user;
	}

	private generateId(): string {
		return Math.random().toString(36).substr(2, 9);
	}
}

// Namespace
namespace Validation {
	export interface Rule<T> {
		validate(value: T): boolean;
		message: string;
	}

	export class EmailRule implements Rule<string> {
		message = "Invalid email format";

		validate(value: string): boolean {
			return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
		}
	}

	export class MinLengthRule implements Rule<string> {
		constructor(private minLength: number) {}

		get message(): string {
			return `Minimum length is ${this.minLength}`;
		}

		validate(value: string): boolean {
			return value.length >= this.minLength;
		}
	}

	export class Validator<T> {
		private rules: Rule<T>[] = [];

		addRule(rule: Rule<T>): this {
			this.rules.push(rule);
			return this;
		}

		validate(value: T): { isValid: boolean; errors: string[] } {
			const errors: string[] = [];

			for (const rule of this.rules) {
				if (!rule.validate(value)) {
					errors.push(rule.message);
				}
			}

			return {
				isValid: errors.length === 0,
				errors,
			};
		}
	}
}

// Module augmentation
declare global {
	interface Array<T> {
		groupBy<K extends string | number | symbol>(
			keyFn: (item: T) => K,
		): Record<K, T[]>;
	}
}

// Complex async function with generics
async function fetchWithRetry<T>(
	url: string,
	options: RequestInit = {},
	maxRetries: number = 3,
): Promise<ApiResponse<T>> {
	let lastError: Error = new Error("No attempts made");

	for (let attempt = 1; attempt <= maxRetries; attempt++) {
		try {
			const response = await fetch(url, options);

			if (!response.ok) {
				throw new Error(
					`HTTP ${response.status}: ${response.statusText}`,
				);
			}

			const data: T = await response.json();

			return {
				data,
				success: true,
			};
		} catch (error) {
			lastError =
				error instanceof Error ? error : new Error(String(error));

			if (attempt === maxRetries) {
				break;
			}

			// Exponential backoff
			await new Promise((resolve) =>
				setTimeout(resolve, Math.pow(2, attempt) * 1000),
			);
		}
	}

	return {
		data: null as any,
		success: false,
		message: `Failed after ${maxRetries} attempts`,
		errors: [lastError.message],
	};
}

// Type guards
function isUser(obj: any): obj is User {
	return (
		obj &&
		typeof obj === "object" &&
		typeof obj.id !== "undefined" &&
		typeof obj.name === "string" &&
		typeof obj.email === "string" &&
		typeof obj.age === "number" &&
		Array.isArray(obj.roles)
	);
}

function isString(value: unknown): value is string {
	return typeof value === "string";
}

// Assertion functions
function assertIsUser(obj: unknown): asserts obj is User {
	if (!isUser(obj)) {
		throw new Error("Object is not a valid User");
	}
}

// Export statements
export default UserService;

export {
	BaseService,
	ApiController,
	Validation,
	fetchWithRetry,
	isUser,
	isString,
	assertIsUser,
	identity,
	firstElement,
	mapArray,
	processValue,
};

export type {
	ID,
	Status,
	EventHandler,
	HttpMethod,
	ApiEndpoints,
	BaseEntity,
	User,
	UserProfile,
	UserPreferences,
	Role,
	Permission,
	Repository,
	ApiResponse,
	PaginatedResponse,
};

export { UserRole, HttpStatusCode, Direction };
