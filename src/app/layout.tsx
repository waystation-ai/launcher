import "@/app/globals.css";
import { sora } from './fonts';

import { PostHogProvider } from './providers';

import BodyBackground from '@/app/components/BodyBackground';
import Navigation from '@/app/components/Navigation';
import Footer from '@/app/components/Footer';

export default function RootLayout({children}: Readonly<{children: React.ReactNode;}>) {
  return (
    <html lang="en">
      <body className={`${sora.className} antialiased flex flex-col`}>
        <PostHogProvider>
          <BodyBackground/>
          <header className="bg-white/80 rounded-bl-2xl rounded-br-2xl shadow-[0px_2px_16px_0px_rgba(0,0,0,0.08)] border-b border-white backdrop-blur-xl px-4 sm:px-6 py-4 flex flex-row justify-between items-center sticky top-0 z-50 gap-4 sm:gap-0">
            <Navigation/>
          </header>
          <div className="flex-grow">
            {children}
          </div>
          <Footer />
        </PostHogProvider>
      </body>
    </html>
  );
}
