import type { LocalInstanceResponse } from "../types/LocalInstanceResponse";
import { ENDPOINTS } from "../utils/api";
import { json } from "../utils/response";

// Public endpoint, so no auth header: it's read before anyone can sign in.
export const getLocalInstance = (): Promise<LocalInstanceResponse> =>
  fetch(ENDPOINTS.instance.get()).then(json<LocalInstanceResponse>);
