import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Routes, Route } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import './index.css'
import App from './App.tsx'
import HomePage from './pages/HomePage.tsx'
import DossierListPage from './pages/DossierListPage.tsx'
import DossierDetailPage from './pages/DossierDetailPage.tsx'
import GroupListPage from './pages/GroupListPage.tsx'
import GroupDetailPage from './pages/GroupDetailPage.tsx'
import GroupVotesPage from './pages/GroupVotesPage.tsx'
import ScrutinListPage from './pages/ScrutinListPage.tsx'
import ScrutinDetailPage from './pages/ScrutinDetailPage.tsx'
import ThemeListPage from './pages/ThemeListPage.tsx'
import ThemeDetailPage from './pages/ThemeDetailPage.tsx'
import ThemeMethodPage from './pages/ThemeMethodPage.tsx'
import ThemeArbitrationPage from './pages/ThemeArbitrationPage.tsx'
import UnassignedTextsPage from './pages/UnassignedTextsPage.tsx'
import TextDetailPage from './pages/TextDetailPage.tsx'
import ComprendrePage from './pages/ComprendrePage.tsx'

const queryClient = new QueryClient()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route element={<App />}>
            <Route index element={<HomePage />} />
            <Route path="dossiers" element={<DossierListPage />} />
            <Route path="dossiers/:uid" element={<DossierDetailPage />} />
            <Route path="groupes" element={<GroupListPage />} />
            <Route path="groupes/:uid" element={<GroupDetailPage />} />
            <Route path="votes-par-groupe" element={<GroupVotesPage />} />
            <Route path="scrutins" element={<ScrutinListPage />} />
            <Route path="scrutins/:uid" element={<ScrutinDetailPage />} />
            <Route path="themes" element={<ThemeListPage />} />
            <Route path="themes/methode" element={<ThemeMethodPage />} />
            <Route path="themes/non-rattaches" element={<UnassignedTextsPage />} />
            <Route path="themes/arbitrage" element={<ThemeArbitrationPage />} />
            <Route path="themes/:code" element={<ThemeDetailPage />} />
            <Route path="textes/:key" element={<TextDetailPage />} />
            <Route path="comprendre" element={<ComprendrePage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  </StrictMode>,
)
