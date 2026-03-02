import { Navbar } from './components/Navbar'
import { Hero } from './sections/Hero'
import { Features } from './sections/Features'
import { Screenshots } from './sections/Screenshots'
import { Download } from './sections/Download'
import { Footer } from './sections/Footer'

export default function App() {
  return (
    <div className="min-h-screen">
      <Navbar />
      <Hero />
      <Features />
      <Screenshots />
      <Download />
      <Footer />
    </div>
  )
}
