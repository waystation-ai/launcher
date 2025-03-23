import "@/app/globals.css";
import { sora } from './fonts';

import { PostHogProvider } from './providers';

import BodyBackground from '@/app/components/BodyBackground';
import ConditionalHeader from '@/app/components/ConditionalHeader';

export default function RootLayout({children}: Readonly<{children: React.ReactNode;}>) {
  return (
    <html lang="en">
      <body className={`${sora.className} antialiased flex flex-col`}>
        <PostHogProvider>
          <BodyBackground/>
          <ConditionalHeader />
          <div className="flex-grow">
            {children}
          </div>
        </PostHogProvider>
      </body>
    </html>
  );
}
