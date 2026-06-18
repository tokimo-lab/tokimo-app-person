export interface PersonDto {
  id: string;
  name: string;
  face_count: number;
  media_count: number;
  created_at: string;
  updated_at: string;
}

export interface PersonsListResp {
  items: PersonDto[];
  total: number;
}

export interface PersonDetailDto {
  id: string;
  name: string;
  face_count: number;
  media_count: number;
  faces: FaceDto[];
  media: MediaDto[];
  created_at: string;
  updated_at: string;
}

export interface FaceDto {
  id: string;
  media_id: string;
  x: number;
  y: number;
  w: number;
  h: number;
  confidence: number;
  thumbnail_path: string | null;
}

export interface MediaDto {
  id: string;
  path: string;
  thumbnail_path: string | null;
  created_at: string;
}

export interface MatchFaceResp {
  person_id: string | null;
  is_new: boolean;
  similarity: number;
}

export interface RegisterFacesResp {
  cached: number;
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

  getPerson: (id: string) => request<PersonDetailDto>(`/persons/${id}`),

  updatePerson: (id: string, data: { name?: string }) =>
    request<PersonDetailDto>(`/persons/${id}`, {
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
    faces: Array<{ index: number; bbox: [number, number, number, number] }>;
  }) =>
    request<RegisterFacesResp>("/register-faces", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    }),

  deleteSource: (params: { source_app: string; source_id: string }) =>
    request<{ deleted: number }>("/delete-source", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    }),
};
