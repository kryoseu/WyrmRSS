import { handleUnauthorized } from "./auth";
import { throwApiError } from "./error";

export async function json<T>(res: Response): Promise<T> {
  isUnauthorized(res);
  if (!res.ok) await throwApiError(res);
  return res.json();
}

export async function noContent(res: Response): Promise<void> {
  isUnauthorized(res);
  if (!res.ok) await throwApiError(res);
}

export async function blob(res: Response): Promise<Blob> {
  isUnauthorized(res);
  if (!res.ok) await throwApiError(res);
  return res.blob();
}

function isUnauthorized(res: Response) {
  if (res.status === 401) {
    handleUnauthorized();
    throw new Error("Unauthorized");
  }
}
