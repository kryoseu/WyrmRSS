import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import "./App.css";
import { ReaderPage } from "./pages/ReaderPage";
import { PostList } from "./components/PostList";

const qc = new QueryClient();

export default function App() {
  return (
    <QueryClientProvider client={qc}>
      <BrowserRouter>
        <Routes>
          <Route element={<ReaderPage />}>
            <Route path="/" element={<Navigate to="/feeds" replace />} />
            <Route path="/feeds" element={<PostList />} />
            <Route path="/feeds/posts/:postId" element={<PostList />} />
            <Route path="/feeds/:feedId" element={<PostList />} />
            <Route path="/feeds/:feedId/posts/:postId" element={<PostList />} />
            <Route path="/favorites" element={<PostList />} />
            <Route path="/favorites/:postId" element={<PostList />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
