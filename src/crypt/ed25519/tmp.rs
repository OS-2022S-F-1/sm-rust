pub fn madd(ge_p1p1 *r, const ge_p3 *p, const ge_precomp *q) {
fe t0;
fe_add(r->X, p->Y, p->X);
fe_sub(r->Y, p->Y, p->X);
fe_mul(r->Z, r->X, q->yplusx);
fe_mul(r->Y, r->Y, q->yminusx);
fe_mul(r->T, q->xy2d, p->T);
fe_add(t0, p->Z, p->Z);
fe_sub(r->X, r->Z, r->Y);
fe_add(r->Y, r->Z, r->Y);
fe_add(r->Z, t0, r->T);
fe_sub(r->T, t0, r->T);
}


/*
r = p - q
*/

pub fn msub(ge_p1p1 *r, const ge_p3 *p, const ge_precomp *q) {
fe t0;

fe_add(r->X, p->Y, p->X);
fe_sub(r->Y, p->Y, p->X);
fe_mul(r->Z, r->X, q->yminusx);
fe_mul(r->Y, r->Y, q->yplusx);
fe_mul(r->T, q->xy2d, p->T);
fe_add(t0, p->Z, p->Z);
fe_sub(r->X, r->Z, r->Y);
fe_add(r->Y, r->Z, r->Y);
fe_sub(r->Z, t0, r->T);
fe_add(r->T, t0, r->T);
}