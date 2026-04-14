import { createContext, useContext, useState, ReactNode } from "react";

export type Locale = "en" | "ru";

const translations = {
  en: {
    // Nav
    "nav.today": "Today",
    "nav.review": "Review",
    "nav.projects": "Projects",
    "nav.jira": "Jira",
    "nav.settings": "Settings",

    // Tracking states
    "state.running": "Running",
    "state.paused": "Paused",
    "state.idle": "Idle",

    // Actions
    "action.pauseTracking": "Pause tracking",
    "action.resumeTracking": "Resume tracking",
    "action.refresh": "Refresh",
    "action.save": "Save",
    "action.cancel": "Cancel",

    // Layout
    "layout.collapseNav": "Collapse navigation",
    "layout.expandNav": "Expand navigation",

    // Dashboard
    "dashboard.title": "Today",
    "dashboard.totalTracked": "Total tracked",
    "dashboard.sessions": "Sessions",
    "dashboard.issues": "Issues",
    "dashboard.byIssue": "By Issue",
    "dashboard.loading": "Loading\u2026",
    "dashboard.noSessions": "No sessions recorded today.",
    "dashboard.sessionsTitle": "Sessions",
    "dashboard.idle": "idle {duration}",
    "dashboard.published": "\u2713 Published",
    "dashboard.liveSession": "Current Session",
    "dashboard.elapsed": "Elapsed",
    "dashboard.noWindow": "No active window",
    "dashboard.addNote": "Add note…",
    "dashboard.notePlaceholder": "What are you working on?",

    // Review
    "review.title": "Review",
    "review.subtitle": "Edit and publish worklogs to Jira",
    "review.total": "Total",
    "review.unpublished": "Unpublished",
    "review.publishable": "Publishable",
    "review.publishableSessions": "{count} sessions",
    "review.noSessions": "No sessions for this day.",
    "review.colTime": "Time",
    "review.colDuration": "Duration",
    "review.colIssue": "Issue",
    "review.colBranch": "Branch",
    "review.colProject": "Project",
    "review.colNotes": "Notes",
    "review.colStatus": "Status",
    "review.statusPublished": "\u2713 Published",
    "review.statusPending": "Pending",
    "review.noJiraKey": "No Jira key \u2013 edit the session first.",
    "review.publishedSuccess": "Published {duration} to {key}",
    "review.deleteConfirm": "Delete this session?",
    "review.titlePublish": "Publish to Jira",
    "review.titleEdit": "Edit",
    "review.titleDelete": "Delete",
    "review.titleSave": "Save",
    "review.titleCancel": "Cancel",

    // Projects
    "projects.title": "Projects",
    "projects.subtitle": "Register local Git repositories to track",
    "projects.addProject": "Add Project",
    "projects.newProject": "New Project",
    "projects.fieldName": "Name",
    "projects.fieldPath": "Folder path",
    "projects.fieldColor": "Color",
    "projects.noProjects": "No projects registered yet.",
    "projects.noProjectsHint": "Add a folder to start tracking time against it.",
    "projects.deleteConfirm": "Delete project \"{name}\"? Sessions will remain but lose their project link.",
    "projects.browsePath": "Browse folder",

    // Jira Settings
    "jira.title": "Jira Connection",
    "jira.subtitle": "Configure Jira Cloud for worklog publishing",
    "jira.connectedInfo": "Connected to {url} as {email}. Enter a new API token below to update.",
    "jira.connectionDetails": "Connection details",
    "jira.displayName": "Display name",
    "jira.baseUrl": "Jira base URL",
    "jira.email": "Email",
    "jira.apiToken": "API Token",
    "jira.apiTokenPlaceholder": "Paste token from Atlassian account settings",
    "jira.apiTokenUpdatePlaceholder": "Enter new token to update",
    "jira.tokenHint": "Generate at: Account Settings \u2192 Security \u2192 API tokens",
    "jira.save": "Save",
    "jira.testConnection": "Test connection",
    "jira.howItWorks": "How it works",
    "jira.step1": "TimeTracker detects the active Git branch every few seconds.",
    "jira.step2": "A Jira key (e.g. PROJ-123) is parsed from the branch name.",
    "jira.step3": "Sessions are recorded locally and never sent to Jira automatically.",
    "jira.step4": "Go to Review to inspect and selectively publish worklogs.",
    "jira.connectedAs": "Connected as: {user}",

    // App Settings
    "appSettings.title": "App Settings",
    "appSettings.subtitle": "Configure tracking behavior and Jira key patterns",
    "appSettings.saved": "Settings saved.",
    "appSettings.tracking": "Tracking",
    "appSettings.idleThreshold": "Idle threshold (seconds)",
    "appSettings.idleHint": "Sessions pause after this many idle seconds.",
    "appSettings.pollInterval": "Poll interval (seconds)",
    "appSettings.pollHint": "How often to sample the active window. Min 2.",
    "appSettings.minimizeToTray": "Minimize to system tray on close",
    "appSettings.jiraPatterns": "Jira Key Patterns",
    "appSettings.patternsHint": "Regex patterns used to extract a Jira key from a Git branch name. Each pattern must have a capture group (\u2026) around the key.",
    "appSettings.addPattern": "Add pattern",
    "appSettings.saveSettings": "Save Settings",
    "appSettings.loading": "Loading\u2026",
    "appSettings.patternDesc": "Description",
    "appSettings.patternTestLabel": "Test branch name",
    "appSettings.patternTestPlaceholder": "feature/PROJ-123-my-feature",
    "appSettings.patternMatches": "Matched: {key}",
    "appSettings.patternNoMatch": "No match",

    // Huddle (Slack call)
    "huddle.live": "Huddle",
    "huddle.in": "in #{channel}",
    "huddle.with": "with {channel}",
    "huddle.elapsed": "Call time",
    "huddle.badge": "Huddle",

    // Manual tracking
    "manual.placeholder": "What are you working on?",
    "manual.add": "Track",
    "manual.stop": "Stop",
    "manual.rename": "Rename",
    "manual.labelEmpty": "Label cannot be empty",

    // Close confirmation
    "close.title": "Active sessions",
    "close.body": "You have active tracking sessions. What would you like to do?",
    "close.minimize": "Minimize to tray",
    "close.stopAndMinimize": "Stop tracking & minimize",
    "close.cancel": "Cancel",

    // App Settings — Integrations section
    "appSettings.integrations": "Integrations",
    "appSettings.slackHuddle": "Track Slack Huddles",
    "appSettings.slackHuddleHint": "Record Slack Huddle calls as sessions",

    // App Settings — Boards section
    "appSettings.boards": "Boards",
    "appSettings.jiraEnable": "Enable Jira integration",
    "appSettings.jiraEnableHint": "Publish worklogs to Jira Cloud from the Review page",

    // App Settings — Storage section
    "appSettings.storage": "Storage",
    "appSettings.storageDb": "Database size",
    "appSettings.storageSessions": "Recorded sessions",
    "appSettings.eraseData": "Erase all sessions",
    "appSettings.eraseConfirm": "Delete all {count} sessions? This cannot be undone.",
    "appSettings.loadingStorage": "Loading storage info\u2026",
    "appSettings.erased": "All sessions erased.",

    // Language toggle label (shown to switch TO that lang)
    "lang.switchTo": "RU",
  },

  ru: {
    // Nav
    "nav.today": "\u0421\u0435\u0433\u043e\u0434\u043d\u044f",
    "nav.review": "\u041e\u0431\u0437\u043e\u0440",
    "nav.projects": "\u041f\u0440\u043e\u0435\u043a\u0442\u044b",
    "nav.jira": "Jira",
    "nav.settings": "\u041d\u0430\u0441\u0442\u0440\u043e\u0439\u043a\u0438",

    // Tracking states
    "state.running": "\u0410\u043a\u0442\u0438\u0432\u0435\u043d",
    "state.paused": "\u041f\u0430\u0443\u0437\u0430",
    "state.idle": "\u041f\u0440\u043e\u0441\u0442\u043e\u0439",

    // Actions
    "action.pauseTracking": "\u041f\u0430\u0443\u0437\u0430",
    "action.resumeTracking": "\u041f\u0440\u043e\u0434\u043e\u043b\u0436\u0438\u0442\u044c",
    "action.refresh": "\u041e\u0431\u043d\u043e\u0432\u0438\u0442\u044c",
    "action.save": "\u0421\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c",
    "action.cancel": "\u041e\u0442\u043c\u0435\u043d\u0430",

    // Layout
    "layout.collapseNav": "\u0421\u0432\u0435\u0440\u043d\u0443\u0442\u044c",
    "layout.expandNav": "\u0420\u0430\u0437\u0432\u0435\u0440\u043d\u0443\u0442\u044c",

    // Dashboard
    "dashboard.title": "\u0421\u0435\u0433\u043e\u0434\u043d\u044f",
    "dashboard.totalTracked": "\u0412\u0441\u0435\u0433\u043e \u043e\u0442\u0441\u043b\u0435\u0436\u0435\u043d\u043e",
    "dashboard.sessions": "\u0421\u0435\u0441\u0441\u0438\u0439",
    "dashboard.issues": "\u0417\u0430\u0434\u0430\u0447",
    "dashboard.byIssue": "\u041f\u043e \u0437\u0430\u0434\u0430\u0447\u0430\u043c",
    "dashboard.loading": "\u0417\u0430\u0433\u0440\u0443\u0437\u043a\u0430\u2026",
    "dashboard.noSessions": "\u0421\u0435\u0441\u0441\u0438\u0439 \u0437\u0430 \u0441\u0435\u0433\u043e\u0434\u043d\u044f \u043d\u0435 \u0437\u0430\u043f\u0438\u0441\u0430\u043d\u043e.",
    "dashboard.sessionsTitle": "\u0421\u0435\u0441\u0441\u0438\u0438",
    "dashboard.idle": "\u043f\u0440\u043e\u0441\u0442\u043e\u0439 {duration}",
    "dashboard.published": "\u2713 \u041e\u043f\u0443\u0431\u043b\u0438\u043a\u043e\u0432\u0430\u043d\u043e",
    "dashboard.liveSession": "\u0422\u0435\u043a\u0443\u0449\u0430\u044f \u0441\u0435\u0441\u0441\u0438\u044f",
    "dashboard.elapsed": "\u041f\u0440\u043e\u0448\u043b\u043e",
    "dashboard.noWindow": "\u041d\u0435\u0442 \u0430\u043a\u0442\u0438\u0432\u043d\u043e\u0433\u043e \u043e\u043a\u043d\u0430",
    "dashboard.addNote": "\u0414\u043e\u0431\u0430\u0432\u0438\u0442\u044c \u0437\u0430\u043c\u0435\u0442\u043a\u0443\u2026",
    "dashboard.notePlaceholder": "\u041d\u0430\u0434 \u0447\u0435\u043c \u0440\u0430\u0431\u043e\u0442\u0430\u0435\u0442\u0435?",

    // Review
    "review.title": "\u041e\u0431\u0437\u043e\u0440",
    "review.subtitle": "\u0420\u0435\u0434\u0430\u043a\u0442\u0438\u0440\u043e\u0432\u0430\u043d\u0438\u0435 \u0438 \u043f\u0443\u0431\u043b\u0438\u043a\u0430\u0446\u0438\u044f \u0432\u043e\u0440\u043a\u043b\u043e\u0433\u043e\u0432 \u0432 Jira",
    "review.total": "\u0412\u0441\u0435\u0433\u043e",
    "review.unpublished": "\u041d\u0435 \u043e\u043f\u0443\u0431\u043b\u0438\u043a\u043e\u0432\u0430\u043d\u043e",
    "review.publishable": "\u041a \u043f\u0443\u0431\u043b\u0438\u043a\u0430\u0446\u0438\u0438",
    "review.publishableSessions": "{count} \u0441\u0435\u0441\u0441\u0438\u0439",
    "review.noSessions": "\u0421\u0435\u0441\u0441\u0438\u0439 \u0437\u0430 \u044d\u0442\u043e\u0442 \u0434\u0435\u043d\u044c \u043d\u0435\u0442.",
    "review.colTime": "\u0412\u0440\u0435\u043c\u044f",
    "review.colDuration": "\u0414\u043b\u0438\u0442.",
    "review.colIssue": "\u0417\u0430\u0434\u0430\u0447\u0430",
    "review.colBranch": "\u0412\u0435\u0442\u043a\u0430",
    "review.colProject": "\u041f\u0440\u043e\u0435\u043a\u0442",
    "review.colNotes": "\u0417\u0430\u043c\u0435\u0442\u043a\u0438",
    "review.colStatus": "\u0421\u0442\u0430\u0442\u0443\u0441",
    "review.statusPublished": "\u2713 \u041e\u043f\u0443\u0431\u043b\u0438\u043a\u043e\u0432\u0430\u043d\u043e",
    "review.statusPending": "\u041e\u0436\u0438\u0434\u0430\u0435\u0442",
    "review.noJiraKey": "\u041d\u0435\u0442 \u043a\u043b\u044e\u0447\u0430 Jira \u2014 \u0441\u043d\u0430\u0447\u0430\u043b\u0430 \u043e\u0442\u0440\u0435\u0434\u0430\u043a\u0442\u0438\u0440\u0443\u0439\u0442\u0435 \u0441\u0435\u0441\u0441\u0438\u044e.",
    "review.publishedSuccess": "\u041e\u043f\u0443\u0431\u043b\u0438\u043a\u043e\u0432\u0430\u043d\u043e {duration} \u0432 {key}",
    "review.deleteConfirm": "\u0423\u0434\u0430\u043b\u0438\u0442\u044c \u044d\u0442\u0443 \u0441\u0435\u0441\u0441\u0438\u044e?",
    "review.titlePublish": "\u041e\u043f\u0443\u0431\u043b\u0438\u043a\u043e\u0432\u0430\u0442\u044c \u0432 Jira",
    "review.titleEdit": "\u0418\u0437\u043c\u0435\u043d\u0438\u0442\u044c",
    "review.titleDelete": "\u0423\u0434\u0430\u043b\u0438\u0442\u044c",
    "review.titleSave": "\u0421\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c",
    "review.titleCancel": "\u041e\u0442\u043c\u0435\u043d\u0430",

    // Projects
    "projects.title": "\u041f\u0440\u043e\u0435\u043a\u0442\u044b",
    "projects.subtitle": "\u0420\u0435\u0433\u0438\u0441\u0442\u0440\u0430\u0446\u0438\u044f Git-\u0440\u0435\u043f\u043e\u0437\u0438\u0442\u043e\u0440\u0438\u0435\u0432 \u0434\u043b\u044f \u043e\u0442\u0441\u043b\u0435\u0436\u0438\u0432\u0430\u043d\u0438\u044f",
    "projects.addProject": "\u0414\u043e\u0431\u0430\u0432\u0438\u0442\u044c \u043f\u0440\u043e\u0435\u043a\u0442",
    "projects.newProject": "\u041d\u043e\u0432\u044b\u0439 \u043f\u0440\u043e\u0435\u043a\u0442",
    "projects.fieldName": "\u041d\u0430\u0437\u0432\u0430\u043d\u0438\u0435",
    "projects.fieldPath": "\u041f\u0443\u0442\u044c \u043a \u043f\u0430\u043f\u043a\u0435",
    "projects.fieldColor": "\u0426\u0432\u0435\u0442",
    "projects.noProjects": "\u041f\u0440\u043e\u0435\u043a\u0442\u044b \u0435\u0449\u0451 \u043d\u0435 \u0437\u0430\u0440\u0435\u0433\u0438\u0441\u0442\u0440\u0438\u0440\u043e\u0432\u0430\u043d\u044b.",
    "projects.noProjectsHint": "\u0414\u043e\u0431\u0430\u0432\u044c\u0442\u0435 \u043f\u0430\u043f\u043a\u0443, \u0447\u0442\u043e\u0431\u044b \u043d\u0430\u0447\u0430\u0442\u044c \u043e\u0442\u0441\u043b\u0435\u0436\u0438\u0432\u0430\u043d\u0438\u0435.",
    "projects.deleteConfirm": "\u0423\u0434\u0430\u043b\u0438\u0442\u044c \u043f\u0440\u043e\u0435\u043a\u0442 \u00ab{name}\u00bb? \u0421\u0435\u0441\u0441\u0438\u0438 \u0441\u043e\u0445\u0440\u0430\u043d\u044f\u0442\u0441\u044f, \u043d\u043e \u043f\u043e\u0442\u0435\u0440\u044f\u044e\u0442 \u043f\u0440\u0438\u0432\u044f\u0437\u043a\u0443.",
    "projects.browsePath": "\u0412\u044b\u0431\u0440\u0430\u0442\u044c \u043f\u0430\u043f\u043a\u0443",

    // Jira Settings
    "jira.title": "\u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u0435 Jira",
    "jira.subtitle": "\u041d\u0430\u0441\u0442\u0440\u043e\u0439\u043a\u0430 Jira Cloud \u0434\u043b\u044f \u043f\u0443\u0431\u043b\u0438\u043a\u0430\u0446\u0438\u0438 \u0432\u043e\u0440\u043a\u043b\u043e\u0433\u043e\u0432",
    "jira.connectedInfo": "\u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u043e \u043a {url} \u043a\u0430\u043a {email}. \u0412\u0432\u0435\u0434\u0438\u0442\u0435 \u043d\u043e\u0432\u044b\u0439 API-\u0442\u043e\u043a\u0435\u043d \u043d\u0438\u0436\u0435 \u0434\u043b\u044f \u043e\u0431\u043d\u043e\u0432\u043b\u0435\u043d\u0438\u044f.",
    "jira.connectionDetails": "\u041f\u0430\u0440\u0430\u043c\u0435\u0442\u0440\u044b \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u044f",
    "jira.displayName": "\u041e\u0442\u043e\u0431\u0440\u0430\u0436\u0430\u0435\u043c\u043e\u0435 \u0438\u043c\u044f",
    "jira.baseUrl": "\u0411\u0430\u0437\u043e\u0432\u044b\u0439 URL Jira",
    "jira.email": "Email",
    "jira.apiToken": "API-\u0442\u043e\u043a\u0435\u043d",
    "jira.apiTokenPlaceholder": "\u0412\u0441\u0442\u0430\u0432\u044c\u0442\u0435 \u0442\u043e\u043a\u0435\u043d \u0438\u0437 \u043d\u0430\u0441\u0442\u0440\u043e\u0435\u043a \u0430\u043a\u043a\u0430\u0443\u043d\u0442\u0430 Atlassian",
    "jira.apiTokenUpdatePlaceholder": "\u0412\u0432\u0435\u0434\u0438\u0442\u0435 \u043d\u043e\u0432\u044b\u0439 \u0442\u043e\u043a\u0435\u043d \u0434\u043b\u044f \u043e\u0431\u043d\u043e\u0432\u043b\u0435\u043d\u0438\u044f",
    "jira.tokenHint": "\u0421\u043e\u0437\u0434\u0430\u0442\u044c: \u041d\u0430\u0441\u0442\u0440\u043e\u0439\u043a\u0438 \u0430\u043a\u043a\u0430\u0443\u043d\u0442\u0430 \u2192 \u0411\u0435\u0437\u043e\u043f\u0430\u0441\u043d\u043e\u0441\u0442\u044c \u2192 API-\u0442\u043e\u043a\u0435\u043d\u044b",
    "jira.save": "\u0421\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c",
    "jira.testConnection": "\u041f\u0440\u043e\u0432\u0435\u0440\u0438\u0442\u044c",
    "jira.howItWorks": "\u041a\u0430\u043a \u044d\u0442\u043e \u0440\u0430\u0431\u043e\u0442\u0430\u0435\u0442",
    "jira.step1": "TimeTracker \u043e\u043f\u0440\u0435\u0434\u0435\u043b\u044f\u0435\u0442 \u0430\u043a\u0442\u0438\u0432\u043d\u0443\u044e \u0432\u0435\u0442\u043a\u0443 Git \u043a\u0430\u0436\u0434\u044b\u0435 \u043d\u0435\u0441\u043a\u043e\u043b\u044c\u043a\u043e \u0441\u0435\u043a\u0443\u043d\u0434.",
    "jira.step2": "\u041a\u043b\u044e\u0447 Jira (\u043d\u0430\u043f\u0440. PROJ-123) \u0438\u0437\u0432\u043b\u0435\u043a\u0430\u0435\u0442\u0441\u044f \u0438\u0437 \u0438\u043c\u0435\u043d\u0438 \u0432\u0435\u0442\u043a\u0438.",
    "jira.step3": "\u0421\u0435\u0441\u0441\u0438\u0438 \u0437\u0430\u043f\u0438\u0441\u044b\u0432\u0430\u044e\u0442\u0441\u044f \u043b\u043e\u043a\u0430\u043b\u044c\u043d\u043e \u0438 \u043d\u0435 \u043e\u0442\u043f\u0440\u0430\u0432\u043b\u044f\u044e\u0442\u0441\u044f \u0432 Jira \u0430\u0432\u0442\u043e\u043c\u0430\u0442\u0438\u0447\u0435\u0441\u043a\u0438.",
    "jira.step4": "\u041f\u0435\u0440\u0435\u0439\u0434\u0438\u0442\u0435 \u0432 \u00ab\u041e\u0431\u0437\u043e\u0440\u00bb \u0434\u043b\u044f \u043f\u0440\u043e\u0432\u0435\u0440\u043a\u0438 \u0438 \u0432\u044b\u0431\u043e\u0440\u043e\u0447\u043d\u043e\u0439 \u043f\u0443\u0431\u043b\u0438\u043a\u0430\u0446\u0438\u0438 \u0432\u043e\u0440\u043a\u043b\u043e\u0433\u043e\u0432.",
    "jira.connectedAs": "\u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u043e \u043a\u0430\u043a: {user}",

    // App Settings
    "appSettings.title": "\u041d\u0430\u0441\u0442\u0440\u043e\u0439\u043a\u0438",
    "appSettings.subtitle": "\u041f\u0430\u0440\u0430\u043c\u0435\u0442\u0440\u044b \u043e\u0442\u0441\u043b\u0435\u0436\u0438\u0432\u0430\u043d\u0438\u044f \u0438 \u0448\u0430\u0431\u043b\u043e\u043d\u044b \u043a\u043b\u044e\u0447\u0435\u0439 Jira",
    "appSettings.saved": "\u041d\u0430\u0441\u0442\u0440\u043e\u0439\u043a\u0438 \u0441\u043e\u0445\u0440\u0430\u043d\u0435\u043d\u044b.",
    "appSettings.tracking": "\u041e\u0442\u0441\u043b\u0435\u0436\u0438\u0432\u0430\u043d\u0438\u0435",
    "appSettings.idleThreshold": "\u041f\u043e\u0440\u043e\u0433 \u043f\u0440\u043e\u0441\u0442\u043e\u044f (\u0441\u0435\u043a\u0443\u043d\u0434\u044b)",
    "appSettings.idleHint": "\u0421\u0435\u0441\u0441\u0438\u044f \u043f\u0440\u0438\u043e\u0441\u0442\u0430\u043d\u0430\u0432\u043b\u0438\u0432\u0430\u0435\u0442\u0441\u044f \u043f\u043e\u0441\u043b\u0435 \u0441\u0442\u043e\u043b\u044c\u043a\u0438\u0445 \u0441\u0435\u043a\u0443\u043d\u0434 \u043f\u0440\u043e\u0441\u0442\u043e\u044f.",
    "appSettings.pollInterval": "\u0418\u043d\u0442\u0435\u0440\u0432\u0430\u043b \u043e\u043f\u0440\u043e\u0441\u0430 (\u0441\u0435\u043a\u0443\u043d\u0434\u044b)",
    "appSettings.pollHint": "\u041a\u0430\u043a \u0447\u0430\u0441\u0442\u043e \u043f\u0440\u043e\u0432\u0435\u0440\u044f\u0442\u044c \u0430\u043a\u0442\u0438\u0432\u043d\u043e\u0435 \u043e\u043a\u043d\u043e. \u041c\u0438\u043d. 2.",
    "appSettings.minimizeToTray": "\u0421\u0432\u043e\u0440\u0430\u0447\u0438\u0432\u0430\u0442\u044c \u0432 \u0442\u0440\u0435\u0439 \u043f\u0440\u0438 \u0437\u0430\u043a\u0440\u044b\u0442\u0438\u0438",
    "appSettings.jiraPatterns": "\u0428\u0430\u0431\u043b\u043e\u043d\u044b \u043a\u043b\u044e\u0447\u0435\u0439 Jira",
    "appSettings.patternsHint": "\u0420\u0435\u0433\u0443\u043b\u044f\u0440\u043d\u044b\u0435 \u0432\u044b\u0440\u0430\u0436\u0435\u043d\u0438\u044f \u0434\u043b\u044f \u0438\u0437\u0432\u043b\u0435\u0447\u0435\u043d\u0438\u044f \u043a\u043b\u044e\u0447\u0430 Jira \u0438\u0437 \u0438\u043c\u0435\u043d\u0438 \u0432\u0435\u0442\u043a\u0438 Git. \u041a\u0430\u0436\u0434\u044b\u0439 \u0448\u0430\u0431\u043b\u043e\u043d \u0434\u043e\u043b\u0436\u0435\u043d \u0441\u043e\u0434\u0435\u0440\u0436\u0430\u0442\u044c \u0433\u0440\u0443\u043f\u043f\u0443 \u0437\u0430\u0445\u0432\u0430\u0442\u0430 (\u2026) \u0432\u043e\u043a\u0440\u0443\u0433 \u043a\u043b\u044e\u0447\u0430.",
    "appSettings.addPattern": "\u0414\u043e\u0431\u0430\u0432\u0438\u0442\u044c \u0448\u0430\u0431\u043b\u043e\u043d",
    "appSettings.saveSettings": "\u0421\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c \u043d\u0430\u0441\u0442\u0440\u043e\u0439\u043a\u0438",
    "appSettings.loading": "\u0417\u0430\u0433\u0440\u0443\u0437\u043a\u0430\u2026",
    "appSettings.patternDesc": "\u041e\u043f\u0438\u0441\u0430\u043d\u0438\u0435",
    "appSettings.patternTestLabel": "\u0422\u0435\u0441\u0442\u043e\u0432\u0430\u044f \u0441\u0442\u0440\u043e\u043a\u0430",
    "appSettings.patternTestPlaceholder": "feature/PROJ-123-my-feature",
    "appSettings.patternMatches": "\u0421\u043e\u0432\u043f\u0430\u0434\u0435\u043d\u0438\u0435: {key}",
    "appSettings.patternNoMatch": "\u041d\u0435\u0442 \u0441\u043e\u0432\u043f\u0430\u0434\u0435\u043d\u0438\u0439",

    // Manual tracking
    "manual.placeholder": "\u041d\u0430\u0434 \u0447\u0435\u043c \u0440\u0430\u0431\u043e\u0442\u0430\u0435\u0442\u0435?",
    "manual.add": "\u041e\u0442\u0441\u043b\u0435\u0436\u0438\u0432\u0430\u0442\u044c",
    "manual.stop": "\u0421\u0442\u043e\u043f",
    "manual.rename": "\u041f\u0435\u0440\u0435\u0438\u043c\u0435\u043d\u043e\u0432\u0430\u0442\u044c",
    "manual.labelEmpty": "\u041d\u0430\u0437\u0432\u0430\u043d\u0438\u0435 \u043d\u0435 \u043c\u043e\u0436\u0435\u0442 \u0431\u044b\u0442\u044c \u043f\u0443\u0441\u0442\u044b\u043c",

    // Close confirmation
    "close.title": "\u0410\u043a\u0442\u0438\u0432\u043d\u044b\u0435 \u0441\u0435\u0441\u0441\u0438\u0438",
    "close.body": "\u0415\u0441\u0442\u044c \u0430\u043a\u0442\u0438\u0432\u043d\u044b\u0435 \u0441\u0435\u0441\u0441\u0438\u0438 \u043e\u0442\u0441\u043b\u0435\u0436\u0438\u0432\u0430\u043d\u0438\u044f. \u0427\u0442\u043e \u0441\u0434\u0435\u043b\u0430\u0442\u044c?",
    "close.minimize": "\u0421\u0432\u0435\u0440\u043d\u0443\u0442\u044c \u0432 \u0442\u0440\u0435\u0439",
    "close.stopAndMinimize": "\u041e\u0441\u0442\u0430\u043d\u043e\u0432\u0438\u0442\u044c \u0438 \u0441\u0432\u0435\u0440\u043d\u0443\u0442\u044c",
    "close.cancel": "\u041e\u0442\u043c\u0435\u043d\u0430",

    // Huddle (Slack call)
    "huddle.live": "\u0417\u0432\u043e\u043d\u043e\u043a",
    "huddle.in": "\u0432 #{channel}",
    "huddle.with": "\u0441 {channel}",
    "huddle.elapsed": "\u0414\u043b\u0438\u0442\u0435\u043b\u044c\u043d\u043e\u0441\u0442\u044c",
    "huddle.badge": "Huddle",

    // App Settings — Integrations section
    "appSettings.integrations": "\u0418\u043d\u0442\u0435\u0433\u0440\u0430\u0446\u0438\u0438",
    "appSettings.slackHuddle": "\u041e\u0442\u0441\u043b\u0435\u0436\u0438\u0432\u0430\u0442\u044c Slack Huddle",
    "appSettings.slackHuddleHint": "\u0417\u0430\u043f\u0438\u0441\u044b\u0432\u0430\u0442\u044c \u0437\u0432\u043e\u043d\u043a\u0438 Slack Huddle \u043a\u0430\u043a \u0441\u0435\u0441\u0441\u0438\u0438",

    // App Settings — Boards section
    "appSettings.boards": "\u0414\u043e\u0441\u043a\u0438",
    "appSettings.jiraEnable": "\u0412\u043a\u043b\u044e\u0447\u0438\u0442\u044c \u0438\u043d\u0442\u0435\u0433\u0440\u0430\u0446\u0438\u044e \u0441 Jira",
    "appSettings.jiraEnableHint": "\u041f\u0443\u0431\u043b\u0438\u043a\u043e\u0432\u0430\u0442\u044c \u0432\u043e\u0440\u043a\u043b\u043e\u0433\u0438 \u0432 Jira Cloud \u0441\u043e \u0441\u0442\u0440\u0430\u043d\u0438\u0446\u044b \u041e\u0431\u0437\u043e\u0440",

    // App Settings — Storage section
    "appSettings.storage": "\u0425\u0440\u0430\u043d\u0438\u043b\u0438\u0449\u0435",
    "appSettings.storageDb": "\u0420\u0430\u0437\u043c\u0435\u0440 \u0431\u0430\u0437\u044b \u0434\u0430\u043d\u043d\u044b\u0445",
    "appSettings.storageSessions": "\u0417\u0430\u043f\u0438\u0441\u0430\u043d\u043e \u0441\u0435\u0441\u0441\u0438\u0439",
    "appSettings.eraseData": "\u0423\u0434\u0430\u043b\u0438\u0442\u044c \u0432\u0441\u0435 \u0441\u0435\u0441\u0441\u0438\u0438",
    "appSettings.eraseConfirm": "\u0423\u0434\u0430\u043b\u0438\u0442\u044c {count} \u0441\u0435\u0441\u0441\u0438\u0439? \u042d\u0442\u043e \u0434\u0435\u0439\u0441\u0442\u0432\u0438\u0435 \u043d\u0435\u043b\u044c\u0437\u044f \u043e\u0442\u043c\u0435\u043d\u0438\u0442\u044c.",
    "appSettings.loadingStorage": "\u0417\u0430\u0433\u0440\u0443\u0437\u043a\u0430\u2026",
    "appSettings.erased": "\u0412\u0441\u0435 \u0441\u0435\u0441\u0441\u0438\u0438 \u0443\u0434\u0430\u043b\u0435\u043d\u044b.",

    // Language toggle label
    "lang.switchTo": "EN",
  },
} as const;

export type TranslationKey = keyof typeof translations["en"];

interface I18nContextType {
  locale: Locale;
  setLocale: (l: Locale) => void;
  t: (key: TranslationKey, params?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nContextType | null>(null);

function interpolate(str: string, params?: Record<string, string | number>): string {
  if (!params) return str;
  return str.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? `{${k}}`));
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(() => {
    return (localStorage.getItem("tt-locale") as Locale) ?? "en";
  });

  function setLocale(l: Locale) {
    setLocaleState(l);
    localStorage.setItem("tt-locale", l);
  }

  function t(key: TranslationKey, params?: Record<string, string | number>): string {
    const dict = translations[locale] as Record<string, string>;
    const fallback = translations["en"] as Record<string, string>;
    const str = dict[key] ?? fallback[key] ?? key;
    return interpolate(str, params);
  }

  return (
    <I18nContext.Provider value={{ locale, setLocale, t }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within I18nProvider");
  return ctx;
}
