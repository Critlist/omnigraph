import React, { useState, useEffect, useCallback } from 'react';
import { observer } from 'mobx-react-lite';
import { debounce } from 'lodash';

// Interface declarations
interface User {
  id: number;
  name: string;
  email: string;
  roles: UserRole[];
}

interface UserRole {
  id: string;
  name: string;
  permissions: Permission[];
}

interface Permission {
  action: string;
  resource: string;
  granted: boolean;
}

// Type aliases
type UserStatus = 'active' | 'inactive' | 'pending';
type LoadingState = 'idle' | 'loading' | 'success' | 'error';

// Generic interface
interface ApiResponse<T> {
  data: T;
  message: string;
  status: number;
}

// Enum
enum UserActions {
  CREATE = 'create',
  UPDATE = 'update',
  DELETE = 'delete',
  VIEW = 'view'
}

// Props interface with generics
interface UserComponentProps<T extends User = User> {
  users: T[];
  onUserSelect?: (user: T) => void;
  onUserUpdate?: (user: T) => Promise<void>;
  className?: string;
  isLoading?: boolean;
}

// Custom hook
const useUserData = <T extends User>(initialUsers: T[]) => {
  const [users, setUsers] = useState<T[]>(initialUsers);
  const [loading, setLoading] = useState<LoadingState>('idle');
  const [selectedUser, setSelectedUser] = useState<T | null>(null);

  const fetchUsers = useCallback(async (): Promise<T[]> => {
    setLoading('loading');
    try {
      const response = await fetch('/api/users');
      const data: ApiResponse<T[]> = await response.json();
      setUsers(data.data);
      setLoading('success');
      return data.data;
    } catch (error) {
      setLoading('error');
      console.error('Failed to fetch users:', error);
      throw error;
    }
  }, []);

  const updateUser = useCallback(async (user: T): Promise<void> => {
    try {
      const response = await fetch(`/api/users/${user.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(user)
      });
      
      if (!response.ok) {
        throw new Error('Failed to update user');
      }

      setUsers(prev => prev.map(u => u.id === user.id ? user : u));
    } catch (error) {
      console.error('Error updating user:', error);
      throw error;
    }
  }, []);

  return {
    users,
    loading,
    selectedUser,
    setSelectedUser,
    fetchUsers,
    updateUser
  };
};

// Class-based component with decorators (hypothetical)
class UserService {
  private apiEndpoint: string;

  constructor(apiEndpoint: string) {
    this.apiEndpoint = apiEndpoint;
  }

  @debounce(300)
  async searchUsers(query: string): Promise<User[]> {
    const response = await fetch(`${this.apiEndpoint}/search?q=${query}`);
    const data: ApiResponse<User[]> = await response.json();
    return data.data;
  }

  static getInstance(): UserService {
    return new UserService('/api/users');
  }

  hasPermission(user: User, action: UserActions, resource: string): boolean {
    return user.roles.some(role => 
      role.permissions.some(permission => 
        permission.action === action && 
        permission.resource === resource && 
        permission.granted
      )
    );
  }
}

// Main functional component
const UserComponent = <T extends User = User>({
  users: initialUsers,
  onUserSelect,
  onUserUpdate,
  className = '',
  isLoading = false
}: UserComponentProps<T>) => {
  const {
    users,
    loading,
    selectedUser,
    setSelectedUser,
    fetchUsers,
    updateUser
  } = useUserData<T>(initialUsers);

  const userService = UserService.getInstance();

  // Effect with cleanup
  useEffect(() => {
    const controller = new AbortController();
    
    const loadUsers = async () => {
      try {
        await fetchUsers();
      } catch (error) {
        if (error.name !== 'AbortError') {
          console.error('Failed to load users:', error);
        }
      }
    };

    loadUsers();

    return () => {
      controller.abort();
    };
  }, [fetchUsers]);

  // Event handlers
  const handleUserClick = useCallback((user: T) => {
    setSelectedUser(user);
    onUserSelect?.(user);
  }, [onUserSelect, setSelectedUser]);

  const handleUserUpdate = useCallback(async (user: T) => {
    try {
      await updateUser(user);
      await onUserUpdate?.(user);
    } catch (error) {
      console.error('Failed to update user:', error);
    }
  }, [updateUser, onUserUpdate]);

  // Render helpers
  const renderUserStatus = (status: UserStatus): React.ReactNode => {
    const statusConfig = {
      active: { color: 'green', text: 'Active' },
      inactive: { color: 'red', text: 'Inactive' },
      pending: { color: 'orange', text: 'Pending' }
    };

    const config = statusConfig[status];
    return (
      <span style={{ color: config.color }}>
        {config.text}
      </span>
    );
  };

  const renderUserList = useCallback(() => {
    return users.map(user => (
      <div 
        key={user.id}
        className="user-item"
        onClick={() => handleUserClick(user)}
        style={{
          cursor: 'pointer',
          padding: '10px',
          border: selectedUser?.id === user.id ? '2px solid blue' : '1px solid gray'
        }}
      >
        <h3>{user.name}</h3>
        <p>{user.email}</p>
        <div>
          Roles: {user.roles.map(role => role.name).join(', ')}
        </div>
        <div>
          Can Delete: {userService.hasPermission(user, UserActions.DELETE, 'users') ? 'Yes' : 'No'}
        </div>
      </div>
    ));
  }, [users, selectedUser, handleUserClick, userService]);

  // Conditional rendering
  if (loading === 'loading' || isLoading) {
    return <div className="loading">Loading users...</div>;
  }

  if (loading === 'error') {
    return <div className="error">Failed to load users</div>;
  }

  return (
    <div className={`user-component ${className}`}>
      <h2>Users ({users.length})</h2>
      <div className="user-list">
        {renderUserList()}
      </div>
      {selectedUser && (
        <div className="selected-user">
          <h3>Selected User</h3>
          <pre>{JSON.stringify(selectedUser, null, 2)}</pre>
          <button onClick={() => handleUserUpdate(selectedUser)}>
            Update User
          </button>
        </div>
      )}
    </div>
  );
};

// Export with observer HOC
export default observer(UserComponent);

// Named exports
export { UserService, useUserData, UserActions };
export type { User, UserRole, Permission, UserStatus, LoadingState, ApiResponse, UserComponentProps }; 