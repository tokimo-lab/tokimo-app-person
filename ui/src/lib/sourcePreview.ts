const THUMB_SOURCE_BY_APP: Record<string, string> = {
  photo: "photo",
  movie: "movie",
  tvshow: "tvshow",
  tv_show: "tv_show",
  season: "season",
  episode: "episode",
};

const SOURCE_LABEL: Record<string, string> = {
  photo: "Photo",
  movie: "Movie",
  tvshow: "TV",
  tv_show: "TV",
  season: "Season",
  episode: "Episode",
};

export interface SourcePreview {
  sourceApp: string;
  sourceId: string;
}

export function getSourceThumbnailUrl(source: SourcePreview, size = 240) {
  const entityType = THUMB_SOURCE_BY_APP[source.sourceApp];
  if (!entityType) return null;

  const encodedId = encodeURIComponent(source.sourceId);
  return `/api/thumb/${entityType}/${encodedId}?w=${size}&h=${size}`;
}

export function getSourceLabel(sourceApp: string) {
  return SOURCE_LABEL[sourceApp] ?? sourceApp;
}
