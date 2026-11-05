import { useNavigate, useSearchParams } from "react-router";
import { AddFeedForm } from "../components/feeds/AddFeedForm";

/** Entry point for feed discovery extensions and subscription bookmarklets. */
export function SubscribePage() {
  const [params] = useSearchParams();
  const navigate = useNavigate();

  return (
    <div className="settings-page">
      <div className="settings-content">
        <section className="settings-section">
          <h2 className="settings-section-title">Add Feed</h2>
          <AddFeedForm
            key={params.toString()}
            initialUrl={params.get("url") ?? ""}
            initialTitle={params.get("title") ?? ""}
            onClose={() => navigate("/", { replace: true })}
          />
        </section>
      </div>
    </div>
  );
}
