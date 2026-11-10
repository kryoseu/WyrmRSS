import { createContext } from "react";

export const AuthContext = createContext({
  oidcSession: false,
  signOut: async (): Promise<void> => {},
});
