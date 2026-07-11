/** API failure carrying the status and the server's curated `error` message. */
export class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

export async function throwApiError(res: Response): Promise<never> {
  const body = (await res.json().catch(() => undefined)) as { error?: string } | undefined;
  throw new ApiError(res.status, body?.error ?? `${res.status} ${res.statusText}`);
}

