CREATE TYPE user_role AS ENUM ('super_admin', 'admin', 'instructor', 'student');
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'blocked');
CREATE TYPE enrollment_status AS ENUM ('pending', 'complete', 'cancelled', 'failed');
CREATE TYPE payment_status AS ENUM ('unpaid', 'paid', 'cancelled', 'failed', 'refunded');
CREATE TYPE review_status AS ENUM ('pending', 'approved', 'rejected');
