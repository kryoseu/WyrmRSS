import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import "./App.css";
import { ReaderPage } from "./pages/ReaderPage";

const qc = new QueryClient();

export default function App() {
  return (
    <QueryClientProvider client={qc}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Navigate to="/feeds" replace />} />
          <Route path="/feeds" element={<ReaderPage />} />
          <Route path="/feeds/posts/:postId" element={<ReaderPage />} />
          <Route path="/feeds/:feedId" element={<ReaderPage />} />
          <Route path="/feeds/:feedId/posts/:postId" element={<ReaderPage />} />
          <Route path="/favorites" element={<ReaderPage />} />
          <Route path="/favorites/:postId" element={<ReaderPage />} />
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
