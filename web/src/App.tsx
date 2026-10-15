import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Route, Routes } from "react-router";
import "./App.css";
import { ApiKeyGate } from "./components/layout/ApiKeyGate";
import { ArchiveList } from "./components/archive/ArchiveList";
import { BookmarkedPostList } from "./components/posts/BookmarkedPostList";
import { PostList } from "./components/posts/PostList";
import { RadarBoard } from "./components/feeds/RadarBoard";
import { ThemeProvider } from "./context/ThemeProvider";
import { AppLayout } from "./pages/AppLayout";
import { FeedsPage } from "./pages/FeedsPage";
import { ReaderPage } from "./pages/ReaderPage";
import { SettingsPage } from "./pages/SettingsPage";

// Always send requests instead of pausing them.
const qc = new QueryClient({
  defaultOptions: {
    queries: { networkMode: "always" },
    mutations: { networkMode: "always" },
  },
});

export default function App() {
  return (
    <ThemeProvider>
      <ApiKeyGate>
        <QueryClientProvider client={qc}>
          <BrowserRouter>
            <Routes>
              <Route element={<AppLayout />}>
                <Route element={<ReaderPage />}>
                  <Route path="/" element={<PostList />} />
                  <Route path="/posts/:postId" element={<PostList />} />
                  <Route path="/feeds/:feedId" element={<PostList />} />
                  <Route path="/feeds/:feedId/posts/:postId" element={<PostList />} />
                  <Route path="/read-later" element={<BookmarkedPostList />} />
                  <Route path="/read-later/:postId" element={<BookmarkedPostList />} />
                  <Route path="/archive" element={<ArchiveList />} />
                  <Route path="/archive/:postId" element={<ArchiveList />} />
                </Route>
                <Route path="/feeds" element={<FeedsPage />} />
                <Route path="/radar" element={<RadarBoard />} />
                <Route path="/settings" element={<SettingsPage />} />
              </Route>
            </Routes>
          </BrowserRouter>
        </QueryClientProvider>
      </ApiKeyGate>
    </ThemeProvider>
  );
}
