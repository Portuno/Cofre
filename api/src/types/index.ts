export interface User {
  id: string;
  email: string;
  created_at: string;
}

export interface Vault {
  id: string;
  name: string;
  description?: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface VaultMember {
  vault_id: string;
  user_id: string;
  role: 'owner' | 'member';
  joined_at: string;
}

export interface VaultInvite {
  id: string;
  vault_id: string;
  invited_email: string;
  token: string;
  accepted: boolean;
  created_at: string;
  expires_at: string;
}

export interface ContentItem {
  id: string;
  vault_id: string;
  created_by: string;
  content_type: 'audio' | 'image' | 'link';
  title?: string;
  url: string;
  transcript?: string;
  metadata?: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface Tag {
  id: string;
  vault_id: string;
  name: string;
  is_special: boolean;
  color?: string;
  created_by: string;
  created_at: string;
}

export interface Embedding {
  id: string;
  content_item_id: string;
  embedding: number[];
  model: string;
  created_at: string;
}

export interface GraphNode {
  item: ContentItem;
  edges: GraphEdge[];
}

export interface GraphEdge {
  target_item_id: string;
  shared_tag?: Tag;
  weight: number;
}

export interface ChatResponse {
  chat_reply_text: string;
  referenced_node_ids: string[];
}

export interface ApiError {
  error: {
    code: string;
    message: string;
    details?: Record<string, any>;
  };
}

export interface AuthToken {
  user: User;
  session_token: string;
}
