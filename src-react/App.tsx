import { BrowserRouter, Routes, Route } from "react-router-dom";
import { I18nProvider } from "./lib/i18n";
import { ToastProvider } from "./lib/toast";
import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import Review from "./pages/Review";
import Projects from "./pages/Projects";
import JiraSettings from "./pages/JiraSettings";
import AppSettings from "./pages/AppSettings";
import "./App.css";

export default function App() {
  return (
    <I18nProvider>
      <ToastProvider>
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
