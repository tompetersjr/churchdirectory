export interface Family {
  id: number;
  family_id: string;
  name: string;
  mailing_name?: string;
  address?: string;
  city?: string;
  state?: string;
  zip?: string;
  phone?: string;
  email?: string;
  photo_path?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface Member {
  id: number;
  family_id: number;
  first_name: string;
  last_name: string;
  role?: string;
  birth_date?: string;
  wedding_date?: string;
  phone?: string;
  email?: string;
  photo_path?: string;
  notes?: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface FamilyWithMembers extends Family {
  members: Member[];
}

export interface ImportPreview {
  families: ImportFamilyPreview[];
  total_families: number;
  total_members: number;
  duplicates: DuplicateMatch[];
}

export interface ImportFamilyPreview {
  family_id: string;
  name: string;
  address?: string;
  members: ImportMemberPreview[];
  is_duplicate: boolean;
  existing_family_id?: number;
}

export interface ImportMemberPreview {
  first_name: string;
  last_name: string;
  role?: string;
}

export interface DuplicateMatch {
  import_family_id: string;
  import_name: string;
  existing_id: number;
  existing_name: string;
  match_type: "id" | "name";
}

export interface ImportResult {
  families_created: number;
  families_updated: number;
  members_created: number;
  members_updated: number;
  errors: string[];
}

export type Theme = "system" | "light" | "dark";

export interface Settings {
  church_name: string;
  church_logo_path?: string;
  theme: Theme;
  default_layout: "grid" | "list";
  page_size: "letter" | "a4";
  include_photos: boolean;
  include_contact_info: boolean;
  include_address: boolean;
}

export interface BackupManifest {
  version: string;
  created_at: string;
  app_version: string;
  family_count: number;
  member_count: number;
  photo_count: number;
}

export type PdfLayout = "grid" | "list";
export type PageSize = "letter" | "a4";

export interface PdfOptions {
  layout: PdfLayout;
  page_size: PageSize;
  include_photos: boolean;
  include_contact_info: boolean;
  include_address: boolean;
  include_cover: boolean;
  include_toc: boolean;
  church_name: string;
  church_logo_path?: string;
}
