import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import "./App.css";
import { PostList } from "./components/PostList";
import { ThemeProvider } from "./context/ThemeProvider";
import { AppLayout } from "./pages/AppLayout";
import { ReaderPage } from "./pages/ReaderPage";
import { SettingsPage } from "./pages/SettingsPage";

const qc = new QueryClient();

export default function App() {
  return (
    <ThemeProvider>
    <QueryClientProvider client={qc}>
      <BrowserRouter>
        <Routes>
          <Route element={<AppLayout />}>
            <Route path="/" element={<Navigate to="/feeds" replace />} />
            <Route element={<ReaderPage />}>
              <Route path="/feeds" element={<PostList />} />
              <Route path="/feeds/posts/:postId" element={<PostList />} />
              <Route path="/feeds/:feedId" element={<PostList />} />
              <Route path="/feeds/:feedId/posts/:postId" element={<PostList />} />
              <Route path="/favorites" element={<PostList />} />
              <Route path="/favorites/:postId" element={<PostList />} />
            </Route>
            <Route path="/settings" element={<SettingsPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
    </ThemeProvider>
  );
}
