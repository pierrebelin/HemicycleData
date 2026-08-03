import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Routes, Route } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import './index.css'
import App from './App.tsx'
import DossierListPage from './pages/DossierListPage.tsx'
import DossierDetailPage from './pages/DossierDetailPage.tsx'
import ScrutinListPage from './pages/ScrutinListPage.tsx'
import ScrutinDetailPage from './pages/ScrutinDetailPage.tsx'

const queryClient = new QueryClient()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route element={<App />}>
            <Route index element={<DossierListPage />} />
            <Route path="dossiers/:uid" element={<DossierDetailPage />} />
            <Route path="scrutins" element={<ScrutinListPage />} />
            <Route path="scrutins/:uid" element={<ScrutinDetailPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  </StrictMode>,
)
