## Grilling Round 2: Permission/Security Model Decisions

Based on user input, here are the decisions for the permission/security model in agent-os:

### 1. Guiding Principle for Human Approval
**Decision**: Potential for harm/data loss
- Actions that could modify/delete data, access sensitive information, or cause system harm require approval
- Safe, read-only, or contained operations can proceed automatically
- This principle balances security with usability for agent autonomy

### 2. Terminal Component Permissions
**Decision**: Command-level scoping
- Different permission levels based on specific command types and patterns
- Examples of permission tiers:
  - **Auto-allowed (no approval needed)**: 
    - `ls`, `pwd`, `cat`, `grep`, `find` (read-only file operations)
    - `git status`, `git log`, `git diff` (read-only git operations)
    - `echo`, `which`, `whoami` (basic system info)
  - **Requires approval**:
    - `rm`, `rmdir`, `del` (delete operations)
    - `mv`, `cp` (modify/move operations - approval for dangerous paths)
    - `npm install`, `yarn add`, `pip install` (dependency installation)
    - `git push`, `git reset --hard` (destructive git operations)
    - `sudo`, `su`, `doas` (privilege escalation)
    - Network commands like `curl`, `wget` to external domains (approval for external calls)
- Implementation: Pattern matching / command classification system

### 3. Browser Component Permissions
**Decision**: Domain-based scoping
- Different permission levels based on website domains and content types
- Examples of permission tiers:
  - **Auto-allowed (no approval needed)**:
    - `localhost:*`, `127.0.0.1:*` (local development)
    - `github.com`, `gitlab.com` (common dev platforms - read-only by default)
    - `stackoverflow.com`, `developer.mozilla.org` (documentation sites)
    - CDN domains for common libraries (when accessing known-safe resources)
  - **Requires approval**:
    - Banking/financial sites (obvious security concern)
    - Sites with form submissions containing sensitive data
    - Unknown/new domains (first-time visit requires approval)
    - Sites attempting to download or execute files
    - Domains flagged by security services (if integrated)
- Implementation: Domain matching with configurable rules and potential content analysis

### 4. Permission System Architecture
**Decision**: Interceptor/middleware approach
- Permission checks happen in the agent-to-component call path
- Transparent to both agents and components
- Central PermissionService intercepts component invocation requests
- Components remain focused on their core functionality
- Permissions can be granted at different levels:
  - **Per-action**: Individual component method calls
  - **Per-session**: Time-bound permission grants
  - **Global**: User-configured defaults

### Permission Interface and Flow

**PermissionRequest Type:**
```typescript
interface PermissionRequest {
  component: string;           // e.g., "terminal", "browser"
  action: string;              // e.g., "execute", "navigate", "evaluate"
  args: any[];                 // Arguments to the action
  context?: {                  // Additional context for decision making
    agentId?: string;          // Which agent is requesting
    sessionId?: string;        // Current session if applicable
    timestamp: number;
  };
  // For terminal: args might be ["ls", "-la"]
  // For browser: args might be ["https://github.com"]
}
```

**PermissionResponse Type:**
```typescript
interface PermissionResponse {
  approved: boolean;
  reason?: string;             // Why approved/denied
  expiresAt?: number;          // If time-limited approval
  constraints?: any;           // Additional constraints on the action
}
```

**PermissionService Interface:**
```typescript
interface PermissionService {
  requestPermission(request: PermissionRequest): Promise<PermissionResponse>;
  // Optional: pre-check without actually performing action
  // Optional: batch permissions for related actions
}
```

**Usage in Agent-Component Flow:**
```typescript
// In the agent-to-component call interceptor:
async function invokeComponent<S extends keyof ComponentInstances>(
  componentType: S,
  action: keyof ComponentInstances[S],
  ...args: Parameters<ComponentInstances[S][keyof ComponentInstances[S]>>
): Promise<ReturnType<ComponentInstances[S][keyof ComponentInstances[S]>>> {
  
  // 1. Construct permission request
  const permissionRequest: PermissionRequest = {
    component: componentType,
    action: action as string,
    args: args,
    context: {
      agentId: currentAgentId,
      sessionId: currentSessionId,
      timestamp: Date.now()
    }
  };
  
  // 2. Request permission from the security system
  const permissionResponse = await PermissionService.requestPermission(permissionRequest);
  
  // 3. Handle the response
  if (!permissionResponse.approved) {
    throw new PermissionError(
      `Permission denied for ${componentType}.${action}: ${permissionResponse.reason}`
    );
  }
  
  // 4. If approved, proceed with component invocation
  const component = await ComponentRegistry.getComponent(componentType);
  // Apply any constraints from permission response
  const constrainedArgs = applyPermissionConstraints(args, permissionResponse.constraints);
  
  // 5. Actually invoke the component
  return component[action](...constrainedArgs);
}
```

### Default Permission Policies

**Terminal Default Policies:**
- Read-only file operations: ALLOW
- Read-only git operations: ALLOW
- System information queries: ALLOW
- File modification/deletion: REQUIRES_APPROVAL (with path validation)
- Dependency installation: REQUIRES_APPROVAL
- Network operations to external domains: REQUIRES_APPROVAL
- Privilege escalation: REQUIRES_APPROVAL
- Destructive git operations: REQUIRES_APPROVAL

**Browser Default Policies:**
- Localhost access: ALLOW
- Known documentation/dev sites (read-only): ALLOW
- First-time visit to external domain: REQUIRES_APPROVAL
- Form submission on external sites: REQUIRES_APPROVAL (especially with sensitive fields)
- File downloads: REQUIRES_APPROVAL
- Navigation to known-risk categories: REQUIRES_APPROVAL
- Script execution from untrusted sources: REQUIRES_APPROVAL

### User Override and Configuration

Users should be able to:
1. **Override defaults**: Configure custom permission rules in their config file
2. **Grant temporary permissions**: Approve actions for a time window
3. **Remember choices**: "Always allow this type of action" or "Always deny"
4. **Review permission history**: See what actions were approved/denied
5. **Emergency override**: Temporarily disable permission system (with clear warning)

### Integration Points

1. **Agent Initialization**: Agents receive PermissionService reference
2. **Component Registration**: Components declare their action types for permission mapping
3. **Configuration Loading**: Permission rules loaded from user config
4. **Audit Logging**: All permission requests and decisions logged for review

### Benefits of This Approach

1. **Fine-grained Control**: Different commands/domains have appropriate permission levels
2. **Transparent to Components**: Components don't need to implement security logic
3. **User Configurable**: Policies can be tailored to user preferences and risk tolerance
4. **Clear Audit Trail**: All permission decisions are trackable
5. **Balances Security and Usability**: Safe operations are fast, risky ones get review
6. **Extensible**: New components can plug into the same permission system

This permission model provides a secure foundation for agent-os while allowing agents to operate autonomously for safe, common operations. The interceptor approach keeps the architecture clean, and the scoping by component/action type provides appropriate granularity for decision making.