import { createContext } from "react";

export const AuthContext = createContext({
  signedIn: false,
  signOut: async (): Promise<void> => {},
});
