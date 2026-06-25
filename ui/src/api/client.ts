export interface PersonDto {
  id: string;
  name: string | null;
  avatar_url: string | null;
  face_count: number;
  media_count: number;
  created_at: string;
  updated_at: string;
}

export interface PersonsListResp {
  items: PersonDto[];
  total: number;
}

export interface FaceDetailDto {
  id: string;
  image_hash: string;
  face_index: number;
  bbox: unknown;
  source_app: string;
  source_id: string;
}

export interface SourceMediaDto {
  id: string;
  source_app: string;
  source_id: string;
  created_at: string;
}

export interface PersonDetailDto {
  id: string;
  name: string | null;
  avatar_url: string | null;
  face_count: number;
  media_count: number;
  faces: FaceDetailDto[];
  media: SourceMediaDto[];
  created_at: string;
  updated_at: string;
}

export interface PhotoOutput {
  id: string;
  filename: string;
  path: string;
  width: number | null;
  height: number | null;
  takenAt: string | null;
  thumbnailPath: string | null;
}

export interface PhotosResponse {
  items: PhotoOutput[];
  total: number;
}

export interface MatchFaceResp {
  person_id: string;
  is_new: boolean;
  similarity: number;
}

export interface RegisterFacesResp {
  cached: number;
}

export interface DeleteSourceResp {
  deleted_cache: number;
  deleted_media: number;
  affected_persons: number;
}

const BASE = "/api/apps/person";

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, options);
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error ?? res.statusText);
  }
  return res.json();
}

export const api = {
  listPersons: (params?: { limit?: number; offset?: number }) => {
    const qs = new URLSearchParams();
    if (params?.limit != null) qs.set("limit", String(params.limit));
    if (params?.offset != null) qs.set("offset", String(params.offset));
    const query = qs.toString();
    return request<PersonsListResp>(`/persons${query ? `?${query}` : ""}`);
  },

  getPerson: (id: string) => request<PersonDetailDto>(`/persons/${id}/detail`),

  updatePerson: (id: string, data: { name?: string }) =>
    request<PersonDto>(`/persons/${id}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    }),

  matchFace: (params: { image_hash: string; face_index: number }) =>
    request<MatchFaceResp>("/match-face", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    }),

  registerFaces: (params: {
    image_hash: string;
    source_app: string;
    source_id: string;
    faces: Array<{ index?: number; bbox?: unknown; embedding?: number[]; [key: string]: unknown }>;
  }) =>
    request<RegisterFacesResp>("/register-faces", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    }),

  deleteSource: (params: { source_app: string; source_id: string }) =>
    request<DeleteSourceResp>("/delete-source", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    }),
};
