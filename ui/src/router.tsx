import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { LoginPage } from "./auth/LoginPage";
import { InviteAcceptPage } from "./auth/InviteAcceptPage";
import { WorkspacePickerPage } from "./pages/WorkspacePickerPage";
import { ExtMountPage } from "./pages/ExtMountPage";
import { OfflinePage } from "./pages/OfflinePage";

export function Router() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Navigate to="/workspaces" replace />} />
        <Route path="/login" element={<LoginPage />} />
        <Route path="/invite/:token" element={<InviteAcceptPage />} />
        <Route path="/workspaces" element={<WorkspacePickerPage />} />
        <Route path="/ext/:workspaceId/*" element={<ExtMountPage />} />
        <Route path="*" element={<OfflinePage />} />
      </Routes>
    </BrowserRouter>
  );
}