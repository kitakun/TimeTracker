import { useEffect } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { I18nProvider } from "./lib/i18n";
import { ToastProvider } from "./lib/toast";
import Layout from "./components/Layout";
import GlobalErrorHandler from "./components/GlobalErrorHandler";
import Dashboard from "./pages/Dashboard";
import Review from "./pages/Review";
import Projects from "./pages/Projects";
import JiraSettings from "./pages/JiraSettings";
import AppSettings from "./pages/AppSettings";
import "./App.css";

export default function App() {
  // Close the native splashscreen and reveal the main window once React has
  // mounted and painted its first frame.
  useEffect(() => {
    invoke("close_splashscreen").catch(() => {
      // Ignore — splashscreen may already be closed (e.g. in dev after HMR).
    });
  }, []);

  return (
    <I18nProvider>
      <ToastProvider>
        <GlobalErrorHandler />
        <BrowserRouter>
          <Layout>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/review" element={<Review />} />
              <Route path="/projects" element={<Projects />} />
              <Route path="/jira" element={<JiraSettings />} />
              <Route path="/settings" element={<AppSettings />} />
            </Routes>
          </Layout>
        </BrowserRouter>
      </ToastProvider>
    </I18nProvider>
  );
}
