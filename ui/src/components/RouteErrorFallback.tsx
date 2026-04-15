import { isRouteErrorResponse, Link, useRouteError } from 'react-router-dom'

export function RouteErrorFallback() {
  const err = useRouteError()
  let message = 'Something went wrong.'
  if (isRouteErrorResponse(err)) {
    message = err.statusText || `${err.status}`
  } else if (err instanceof Error) {
    message = err.message
  }
  return (
    <div className="min-h-[50vh] flex flex-col items-center justify-center gap-4 p-8 text-center">
      <p className="text-red-400 text-sm max-w-md" role="alert">
        {message}
      </p>
      <Link to="/" className="text-accent text-sm font-medium hover:underline">
        Back to markets
      </Link>
    </div>
  )
}
