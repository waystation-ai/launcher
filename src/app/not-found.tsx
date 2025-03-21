export default function Home() {

  return (
    <div className="h-screen flex flex-col relative">
      {/* Hero Section */}
      <main className="flex-1 flex flex-col h-full">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 lg:gap-12 px-4 sm:px-8 py-8 max-w-7xl mx-auto h-full">
          {/* Left Column - Branding */}
          <div className="flex items-center justify-center h-full">
            <p className="text-3xl lg:text-4xl text-gray-900 font-bold">404</p>
          </div>

          {/* Right Column - Chat Demo */}
          <div className="flex items-center justify-center h-full">
            <p className="text-lg lg:text-xl text-gray-800 leading-relaxed">AGI Not Found... Yet</p>
          </div>
        </div>
      </main>
    </div>
  );
}
